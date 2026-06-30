//! SQLite 集合仓储实现（精简版）
//!
//! 使用 JSON 列存储子表数据，减少表数量。
//! 树结构通过 parent_id 列表示，根项的 parent_id 为 NULL。

use crate::domain::models::{
    Collection, CollectionIndexItem, CollectionsConfig, CollectionsIndex, FormField, Header, Param,
    SavedResponseIndexEntry, Variable,
};
use crate::domain::repositories::CollectionRepository;
use crate::infrastructure::data_dir;
use crate::infrastructure::sqlite::connection::{with_connection, with_transaction};
use crate::repo_error;
use rusqlite::{params, Connection};
use std::collections::HashMap;
use std::path::PathBuf;

/// SQLite 集合仓储
pub struct SqliteCollectionRepository;

impl SqliteCollectionRepository {
    pub fn new() -> Self {
        Self
    }
}

impl Default for SqliteCollectionRepository {
    fn default() -> Self {
        Self::new()
    }
}

impl SqliteCollectionRepository {
    fn reconstruct_item_from_json(
        id: String,
        name: String,
        description: Option<String>,
        item_type: String,
        _parent_id: Option<String>,
        _order_index: i32,
        method: Option<String>,
        url: Option<String>,
        body: Option<String>,
        body_type: Option<String>,
        params_json: String,
        headers_json: String,
        form_fields_json: String,
        form_files_json: String,
        common_headers_json: String,
        variables_json: String,
        saved_response_ids_json: String,
    ) -> Result<Collection, String> {
        let params_list: Vec<Param> = serde_json::from_str(&params_json)
            .map_err(|e| repo_error!("反序列化参数失败: {}", e))?;

        let headers: Vec<Header> = serde_json::from_str(&headers_json)
            .map_err(|e| repo_error!("反序列化请求头失败: {}", e))?;

        let form_fields: Vec<FormField> = serde_json::from_str(&form_fields_json)
            .map_err(|e| repo_error!("反序列化表单字段失败: {}", e))?;

        let form_files: HashMap<String, Vec<FileInfo>> = serde_json::from_str(&form_files_json)
            .map_err(|e| repo_error!("反序列化表单文件失败: {}", e))?;

        let form_fields_with_files: Vec<FormField> = form_fields
            .into_iter()
            .map(|field| {
                let files = form_files.get(&field.key).cloned();
                FormField {
                    key: field.key,
                    value: field.value,
                    field_type: field.field_type,
                    enabled: field.enabled,
                    files,
                }
            })
            .collect();

        let common_headers: Vec<Header> = serde_json::from_str(&common_headers_json)
            .map_err(|e| repo_error!("反序列化公共请求头失败: {}", e))?;

        let variables: Vec<Variable> = serde_json::from_str(&variables_json)
            .map_err(|e| repo_error!("反序列化集合变量失败: {}", e))?;

        let _saved_response_ids: Vec<String> = serde_json::from_str(&saved_response_ids_json)
            .map_err(|e| repo_error!("反序列化保存响应索引失败: {}", e))?;

        let saved_responses: Vec<SavedResponseIndexEntry> = Vec::new();

        Ok(Collection {
            id,
            name,
            description,
            item_type,
            children: Vec::new(),
            method,
            url,
            params: if params_list.is_empty() {
                None
            } else {
                Some(params_list)
            },
            headers: if headers.is_empty() {
                None
            } else {
                Some(headers)
            },
            body,
            body_type,
            form_fields: if form_fields_with_files.is_empty() {
                None
            } else {
                Some(form_fields_with_files)
            },
            saved_responses: if saved_responses.is_empty() {
                None
            } else {
                Some(saved_responses)
            },
            common_headers: if common_headers.is_empty() {
                None
            } else {
                Some(common_headers)
            },
            collection_variables: if variables.is_empty() {
                None
            } else {
                Some(variables)
            },
            ws_config: None,
        })
    }

    fn build_index_tree(
        flat_items: &Vec<(String, String, String, Option<String>, Option<String>)>,
    ) -> Vec<CollectionIndexItem> {
        let mut children_list: Vec<(Option<String>, usize)> = Vec::new();
        let mut root_indices: Vec<usize> = Vec::new();

        for (idx, (_, _, _, parent_id, _)) in flat_items.iter().enumerate() {
            match parent_id {
                None => root_indices.push(idx),
                Some(pid) => children_list.push((Some(pid.clone()), idx)),
            }
        }

        Self::assemble_index_children_sorted(&root_indices, flat_items, &children_list)
    }

    fn assemble_index_children_sorted(
        indices: &[usize],
        flat_items: &Vec<(String, String, String, Option<String>, Option<String>)>,
        children_list: &Vec<(Option<String>, usize)>,
    ) -> Vec<CollectionIndexItem> {
        let mut result = Vec::with_capacity(indices.len());
        for idx in indices {
            let (id, name, item_type, _parent_id, method) = &flat_items[*idx];

            let child_indices: Vec<usize> = children_list
                .iter()
                .filter_map(|(parent_id, child_idx)| {
                    if parent_id.as_ref() == Some(id) {
                        Some(*child_idx)
                    } else {
                        None
                    }
                })
                .collect();

            let children = if child_indices.is_empty() {
                Vec::new()
            } else {
                Self::assemble_index_children_sorted(&child_indices, flat_items, children_list)
            };

            result.push(CollectionIndexItem {
                id: id.clone(),
                name: name.clone(),
                item_type: item_type.clone(),
                children,
                method: method.clone(),
            });
        }
        result
    }

    fn serialize_item_to_json(
        item: &Collection,
    ) -> Result<(String, String, String, String, String, String, String), String> {
        let params_json = serde_json::to_string(&item.params.as_ref().unwrap_or(&Vec::new()))
            .map_err(|e| repo_error!("序列化参数失败: {}", e))?;

        let headers_json = serde_json::to_string(&item.headers.as_ref().unwrap_or(&Vec::new()))
            .map_err(|e| repo_error!("序列化请求头失败: {}", e))?;

        let form_fields_json =
            serde_json::to_string(&item.form_fields.as_ref().unwrap_or(&Vec::new()))
                .map_err(|e| repo_error!("序列化表单字段失败: {}", e))?;

        let form_files_map: HashMap<String, Vec<FileInfo>> = item
            .form_fields
            .as_ref()
            .unwrap_or(&Vec::new())
            .iter()
            .filter_map(|field| {
                field
                    .files
                    .as_ref()
                    .map(|files| (field.key.clone(), files.clone()))
            })
            .collect();
        let form_files_json = serde_json::to_string(&form_files_map)
            .map_err(|e| repo_error!("序列化表单文件失败: {}", e))?;

        let common_headers_json =
            serde_json::to_string(&item.common_headers.as_ref().unwrap_or(&Vec::new()))
                .map_err(|e| repo_error!("序列化公共请求头失败: {}", e))?;

        let variables_json =
            serde_json::to_string(&item.collection_variables.as_ref().unwrap_or(&Vec::new()))
                .map_err(|e| repo_error!("序列化集合变量失败: {}", e))?;

        let saved_response_ids: Vec<String> = item
            .saved_responses
            .as_ref()
            .unwrap_or(&Vec::new())
            .iter()
            .map(|entry| entry.id.clone())
            .collect();
        let saved_response_ids_json = serde_json::to_string(&saved_response_ids)
            .map_err(|e| repo_error!("序列化保存响应索引失败: {}", e))?;

        Ok((
            params_json,
            headers_json,
            form_fields_json,
            form_files_json,
            common_headers_json,
            variables_json,
            saved_response_ids_json,
        ))
    }

    fn write_collection_tree(
        conn: &Connection,
        items: &[Collection],
        parent_id: Option<&str>,
        start_order: usize,
    ) -> Result<(), String> {
        for (i, item) in items.iter().enumerate() {
            let (
                params_json,
                headers_json,
                form_fields_json,
                form_files_json,
                common_headers_json,
                variables_json,
                saved_response_ids_json,
            ) = Self::serialize_item_to_json(item)?;

            conn.execute(
                "INSERT INTO collection_items \
                 (id, name, description, item_type, parent_id, order_index, \
                  method, url, body, body_type, \
                  params_json, headers_json, form_fields_json, form_files_json, \
                  common_headers_json, variables_json, saved_response_ids_json) \
                 VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?12, ?13, ?14, ?15, ?16, ?17)",
                params![
                    item.id,
                    item.name,
                    item.description,
                    item.item_type,
                    parent_id,
                    start_order + i,
                    item.method,
                    item.url,
                    item.body,
                    item.body_type,
                    params_json,
                    headers_json,
                    form_fields_json,
                    form_files_json,
                    common_headers_json,
                    variables_json,
                    saved_response_ids_json,
                ],
            )
            .map_err(|e| repo_error!("插入集合项失败: {}", e))?;

            if !item.children.is_empty() {
                Self::write_collection_tree(conn, &item.children, Some(&item.id), 0)?;
            }
        }
        Ok(())
    }

    /// 写入集合项（upsert）：存在则 UPDATE 业务列（保留 parent_id / order_index），
    /// 不存在则 INSERT（parent_id / order_index 取默认值，由后续 write_index 修正）。
    fn upsert_item(conn: &Connection, item: &Collection) -> Result<(), String> {
        let (
            params_json,
            headers_json,
            form_fields_json,
            form_files_json,
            common_headers_json,
            variables_json,
            saved_response_ids_json,
        ) = Self::serialize_item_to_json(item)?;

        let updated = conn
            .execute(
                "UPDATE collection_items SET \
                 name = ?2, description = ?3, method = ?4, url = ?5, body = ?6, body_type = ?7, \
                 params_json = ?8, headers_json = ?9, form_fields_json = ?10, form_files_json = ?11, \
                 common_headers_json = ?12, variables_json = ?13, saved_response_ids_json = ?14 \
                 WHERE id = ?1",
                params![
                    item.id,
                    item.name,
                    item.description,
                    item.method,
                    item.url,
                    item.body,
                    item.body_type,
                    params_json,
                    headers_json,
                    form_fields_json,
                    form_files_json,
                    common_headers_json,
                    variables_json,
                    saved_response_ids_json,
                ],
            )
            .map_err(|e| repo_error!("更新集合项失败: {}", e))?;

        if updated == 0 {
            conn.execute(
                "INSERT INTO collection_items \
                 (id, name, description, item_type, parent_id, order_index, \
                  method, url, body, body_type, \
                  params_json, headers_json, form_fields_json, form_files_json, \
                  common_headers_json, variables_json, saved_response_ids_json) \
                 VALUES (?1, ?2, ?3, ?4, NULL, 0, ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?12, ?13, ?14, ?15)",
                params![
                    item.id,
                    item.name,
                    item.description,
                    item.item_type,
                    item.method,
                    item.url,
                    item.body,
                    item.body_type,
                    params_json,
                    headers_json,
                    form_fields_json,
                    form_files_json,
                    common_headers_json,
                    variables_json,
                    saved_response_ids_json,
                ],
            )
            .map_err(|e| repo_error!("插入集合项失败: {}", e))?;
        }
        Ok(())
    }
}

use crate::domain::models::FileInfo;

impl CollectionRepository for SqliteCollectionRepository {
    fn read_index(&self, workspace_id: &str) -> Result<CollectionsIndex, String> {
        let ws = workspace_id.to_string();
        with_connection(&ws, |conn| {
            let mut stmt = conn
                .prepare(
                    "SELECT id, name, item_type, parent_id, method \
                     FROM collection_items \
                     ORDER BY parent_id, order_index",
                )
                .map_err(|e| repo_error!("准备查询集合索引失败: {}", e))?;

            let flat_items: Vec<(String, String, String, Option<String>, Option<String>)> = stmt
                .query_map([], |row| {
                    Ok((
                        row.get::<_, String>(0)?,
                        row.get::<_, String>(1)?,
                        row.get::<_, String>(2)?,
                        row.get::<_, Option<String>>(3)?,
                        row.get::<_, Option<String>>(4)?,
                    ))
                })
                .map_err(|e| repo_error!("查询集合索引失败: {}", e))?
                .collect::<Result<Vec<_>, rusqlite::Error>>()
                .map_err(|e| repo_error!("解析集合索引行数据失败: {}", e))?;

            if flat_items.is_empty() {
                return Ok(CollectionsIndex::default());
            }

            let items = Self::build_index_tree(&flat_items);
            Ok(CollectionsIndex { items })
        })
    }

    fn write_index(&self, workspace_id: &str, index: &CollectionsIndex) -> Result<(), String> {
        let ws = workspace_id.to_string();
        with_connection(&ws, |conn| {
            fn update_order_index(
                items: &[CollectionIndexItem],
                conn: &Connection,
                parent_id: Option<&str>,
            ) -> Result<(), String> {
                for (i, item) in items.iter().enumerate() {
                    conn.execute(
                        "UPDATE collection_items SET order_index = ?1, parent_id = ?2 WHERE id = ?3",
                        params![i as i32, parent_id, item.id],
                    )
                    .map_err(|e| repo_error!("更新排序失败: id={}, {}", item.id, e))?;

                    update_order_index(&item.children, conn, Some(&item.id))?;
                }
                Ok(())
            }

            // 使用 unchecked_transaction（&self 版本），失败时 Drop 自动 ROLLBACK
            let tx = conn
                .unchecked_transaction()
                .map_err(|e| repo_error!("开始事务失败: {}", e))?;

            update_order_index(&index.items, &tx, None)?;

            tx.commit()
                .map_err(|e| repo_error!("提交事务失败: {}", e))?;

            Ok(())
        })
    }

    fn read_item(&self, workspace_id: &str, id: &str) -> Result<Option<Collection>, String> {
        let ws = workspace_id.to_string();
        with_connection(&ws, |conn| {
            let result = conn.query_row(
                "SELECT id, name, description, item_type, parent_id, order_index, \
                        method, url, body, body_type, \
                        params_json, headers_json, form_fields_json, form_files_json, \
                        common_headers_json, variables_json, saved_response_ids_json \
                 FROM collection_items WHERE id = ?1",
                params![id],
                |row| {
                    Ok((
                        row.get::<_, String>(0)?,
                        row.get::<_, String>(1)?,
                        row.get::<_, Option<String>>(2)?,
                        row.get::<_, String>(3)?,
                        row.get::<_, Option<String>>(4)?,
                        row.get::<_, i32>(5)?,
                        row.get::<_, Option<String>>(6)?,
                        row.get::<_, Option<String>>(7)?,
                        row.get::<_, Option<String>>(8)?,
                        row.get::<_, Option<String>>(9)?,
                        row.get::<_, String>(10)?,
                        row.get::<_, String>(11)?,
                        row.get::<_, String>(12)?,
                        row.get::<_, String>(13)?,
                        row.get::<_, String>(14)?,
                        row.get::<_, String>(15)?,
                        row.get::<_, String>(16)?,
                    ))
                },
            );

            match result {
                Ok((
                    id,
                    name,
                    description,
                    item_type,
                    parent_id,
                    order_index,
                    method,
                    url,
                    body,
                    body_type,
                    params_json,
                    headers_json,
                    form_fields_json,
                    form_files_json,
                    common_headers_json,
                    variables_json,
                    saved_response_ids_json,
                )) => Self::reconstruct_item_from_json(
                    id,
                    name,
                    description,
                    item_type,
                    parent_id,
                    order_index,
                    method,
                    url,
                    body,
                    body_type,
                    params_json,
                    headers_json,
                    form_fields_json,
                    form_files_json,
                    common_headers_json,
                    variables_json,
                    saved_response_ids_json,
                )
                .map(Some),
                Err(rusqlite::Error::QueryReturnedNoRows) => Ok(None),
                Err(e) => Err(repo_error!("查询集合项失败: {}", e)),
            }
        })
    }

    fn write_item(&self, workspace_id: &str, item: &Collection) -> Result<(), String> {
        if item.id.is_empty() {
            return Err(repo_error!("集合项 ID 不能为空"));
        }
        if item.name.is_empty() {
            return Err(repo_error!("集合项名称不能为空"));
        }

        let ws = workspace_id.to_string();
        with_transaction(&ws, |conn| {
            Self::upsert_item(conn, item)?;

            Ok(())
        })
    }

    fn write_item_with_index_update(
        &self,
        workspace_id: &str,
        item: &Collection,
    ) -> Result<(), String> {
        if item.id.is_empty() {
            return Err(repo_error!("集合项 ID 不能为空"));
        }
        if item.name.is_empty() {
            return Err(repo_error!("集合项名称不能为空"));
        }

        let ws = workspace_id.to_string();
        with_transaction(&ws, |conn| {
            Self::upsert_item(conn, item)?;

            Ok(())
        })
    }

    fn delete_item(&self, workspace_id: &str, id: &str) -> Result<(), String> {
        let ws = workspace_id.to_string();
        with_connection(&ws, |conn| {
            conn.execute("DELETE FROM collection_items WHERE id = ?1", params![id])
                .map_err(|e| repo_error!("删除集合项失败: {}", e))?;

            Ok(())
        })
    }

    fn delete_item_recursive(&self, workspace_id: &str, item: &Collection) -> Result<(), String> {
        let ws = workspace_id.to_string();
        with_transaction(&ws, |conn| {
            fn collect_ids(item: &Collection) -> Vec<String> {
                let mut ids = vec![item.id.clone()];
                for child in &item.children {
                    ids.extend(collect_ids(child));
                }
                ids
            }

            let all_ids = collect_ids(item);

            for id in &all_ids {
                conn.execute("DELETE FROM collection_items WHERE id = ?1", params![id])
                    .map_err(|e| repo_error!("删除集合项 {} 失败: {}", id, e))?;
            }

            Ok(())
        })
    }

    fn read_all(&self, workspace_id: &str) -> Result<CollectionsConfig, String> {
        let ws = workspace_id.to_string();
        with_connection(&ws, |conn| {
            let mut stmt = conn
                .prepare(
                    "SELECT id, name, description, item_type, parent_id, order_index, \
                        method, url, body, body_type, \
                        params_json, headers_json, form_fields_json, form_files_json, \
                        common_headers_json, variables_json, saved_response_ids_json \
                 FROM collection_items ORDER BY parent_id, order_index",
                )
                .map_err(|e| repo_error!("准备查询所有集合失败: {}", e))?;

            let flat_items: Vec<(
                String,
                String,
                Option<String>,
                String,
                Option<String>,
                i32,
                Option<String>,
                Option<String>,
                Option<String>,
                Option<String>,
                String,
                String,
                String,
                String,
                String,
                String,
                String,
            )> = stmt
                .query_map([], |row| {
                    Ok((
                        row.get::<_, String>(0)?,
                        row.get::<_, String>(1)?,
                        row.get::<_, Option<String>>(2)?,
                        row.get::<_, String>(3)?,
                        row.get::<_, Option<String>>(4)?,
                        row.get::<_, i32>(5)?,
                        row.get::<_, Option<String>>(6)?,
                        row.get::<_, Option<String>>(7)?,
                        row.get::<_, Option<String>>(8)?,
                        row.get::<_, Option<String>>(9)?,
                        row.get::<_, String>(10)?,
                        row.get::<_, String>(11)?,
                        row.get::<_, String>(12)?,
                        row.get::<_, String>(13)?,
                        row.get::<_, String>(14)?,
                        row.get::<_, String>(15)?,
                        row.get::<_, String>(16)?,
                    ))
                })
                .map_err(|e| repo_error!("查询所有集合失败: {}", e))?
                .collect::<Result<Vec<_>, rusqlite::Error>>()
                .map_err(|e| repo_error!("解析集合行数据失败: {}", e))?;

            if flat_items.is_empty() {
                return Ok(CollectionsConfig::default());
            }

            let parsed_items: Vec<(String, Option<String>, Collection)> = flat_items
                .into_iter()
                .map(
                    |(
                        id,
                        name,
                        description,
                        item_type,
                        parent_id,
                        order_index,
                        method,
                        url,
                        body,
                        body_type,
                        params_json,
                        headers_json,
                        form_fields_json,
                        form_files_json,
                        common_headers_json,
                        variables_json,
                        saved_response_ids_json,
                    )| {
                        let collection = Self::reconstruct_item_from_json(
                            id.clone(),
                            name,
                            description,
                            item_type,
                            parent_id.clone(),
                            order_index,
                            method,
                            url,
                            body,
                            body_type,
                            params_json,
                            headers_json,
                            form_fields_json,
                            form_files_json,
                            common_headers_json,
                            variables_json,
                            saved_response_ids_json,
                        )?;
                        Ok((id, parent_id, collection))
                    },
                )
                .collect::<Result<Vec<_>, String>>()?;

            let mut children_list: Vec<(Option<String>, usize)> = Vec::new();
            let mut root_indices: Vec<usize> = Vec::new();

            for (idx, (_, parent_id, _)) in parsed_items.iter().enumerate() {
                match parent_id {
                    None => root_indices.push(idx),
                    Some(pid) => children_list.push((Some(pid.clone()), idx)),
                }
            }

            fn assemble_children_sorted(
                indices: &[usize],
                parsed_items: &Vec<(String, Option<String>, Collection)>,
                children_list: &Vec<(Option<String>, usize)>,
            ) -> Vec<Collection> {
                let mut result = Vec::with_capacity(indices.len());
                for idx in indices {
                    let (_, _, collection) = &parsed_items[*idx];

                    let child_indices: Vec<usize> = children_list
                        .iter()
                        .filter_map(|(parent_id, child_idx)| {
                            if parent_id.as_ref() == Some(&collection.id) {
                                Some(*child_idx)
                            } else {
                                None
                            }
                        })
                        .collect();

                    let children = if child_indices.is_empty() {
                        Vec::new()
                    } else {
                        assemble_children_sorted(&child_indices, parsed_items, children_list)
                    };

                    result.push(Collection {
                        id: collection.id.clone(),
                        name: collection.name.clone(),
                        description: collection.description.clone(),
                        item_type: collection.item_type.clone(),
                        children,
                        method: collection.method.clone(),
                        url: collection.url.clone(),
                        params: collection.params.clone(),
                        headers: collection.headers.clone(),
                        body: collection.body.clone(),
                        body_type: collection.body_type.clone(),
                        form_fields: collection.form_fields.clone(),
                        saved_responses: collection.saved_responses.clone(),
                        common_headers: collection.common_headers.clone(),
                        collection_variables: collection.collection_variables.clone(),
                        ws_config: collection.ws_config.clone(),
                    });
                }
                result
            }

            let collections =
                assemble_children_sorted(&root_indices, &parsed_items, &children_list);
            Ok(CollectionsConfig { collections })
        })
    }

    fn write_all(&self, workspace_id: &str, config: &CollectionsConfig) -> Result<(), String> {
        let ws = workspace_id.to_string();
        with_transaction(&ws, |conn| {
            conn.execute("DELETE FROM collection_items", [])
                .map_err(|e| repo_error!("清空集合表失败: {}", e))?;

            Self::write_collection_tree(conn, &config.collections, None, 0)?;

            Ok(())
        })
    }

    fn get_collections_dir(&self, workspace_id: &str) -> PathBuf {
        data_dir::get_workspace_db_path(workspace_id)
    }

    fn get_index_path(&self, workspace_id: &str) -> PathBuf {
        data_dir::get_workspace_db_path(workspace_id)
    }

    fn get_item_path(&self, workspace_id: &str, _id: &str) -> PathBuf {
        data_dir::get_workspace_db_path(workspace_id)
    }
}
