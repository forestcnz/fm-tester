use crate::domain::models::common::generate_id;
use crate::domain::models::Collection;

/// 集合领域服务
pub struct CollectionDomainService;

impl CollectionDomainService {
    /// 验证集合项
    pub fn validate_collection_item(item: &Collection) -> Result<(), String> {
        item.validate()
    }

    /// 验证集合名称
    pub fn validate_collection_name(name: &str) -> Result<(), String> {
        if name.trim().is_empty() {
            return Err("集合名称不能为空".to_string());
        }
        Ok(())
    }

    /// 生成集合 ID
    pub fn generate_collection_id() -> String {
        generate_id("col")
    }

    /// 生成 API ID
    pub fn generate_api_id() -> String {
        generate_id("api")
    }

    /// 创建集合实体
    pub fn create_collection_entity(name: String, description: Option<String>) -> Collection {
        Collection {
            id: Self::generate_collection_id(),
            name,
            description,
            item_type: "collection".to_string(),
            children: Vec::new(),
            method: None,
            url: None,
            params: None,
            headers: None,
            body: None,
            body_type: None,
            form_fields: None,
            saved_responses: None,
            common_headers: None,
            collection_variables: None,
            ws_config: None,
        }
    }
}

// ==================== 工具函数（从 collection_utils.rs 迁移）====================

/// 查找集合或接口的父链（从根到该节点的路径）
/// 返回所有父集合（包括自身，如果是集合）
pub fn find_ancestor_chain(
    items: &[Collection],
    target_id: &str,
    path: &mut Vec<Collection>,
) -> bool {
    for item in items {
        if item.id == target_id {
            path.push(item.clone());
            return true;
        }
        if find_ancestor_chain(&item.children, target_id, path) {
            path.insert(0, item.clone());
            return true;
        }
    }
    false
}

/// 递归查找集合项
pub fn find_collection_item<'a>(
    items: &'a mut [Collection],
    id: &str,
) -> Option<&'a mut Collection> {
    for item in items.iter_mut() {
        if item.id == id {
            return Some(item);
        }
        if let Some(found) = find_collection_item(&mut item.children, id) {
            return Some(found);
        }
    }
    None
}

/// 递归删除集合项
pub fn remove_collection_item(items: &mut Vec<Collection>, id: &str) -> bool {
    let initial_len = items.len();
    items.retain(|item| item.id != id);
    if items.len() < initial_len {
        return true;
    }
    for item in items.iter_mut() {
        if remove_collection_item(&mut item.children, id) {
            return true;
        }
    }
    false
}

/// 递归查找 API
pub fn find_api_in_collections<'a>(items: &'a [Collection], id: &str) -> Option<&'a Collection> {
    for item in items {
        if item.id == id && item.item_type == "api" {
            return Some(item);
        }
        if let Some(found) = find_api_in_collections(&item.children, id) {
            return Some(found);
        }
    }
    None
}

/// 递归查找集合或 API（不可变版本）
pub fn find_item_in_collections<'a>(items: &'a [Collection], id: &str) -> Option<&'a Collection> {
    for item in items {
        if item.id == id {
            return Some(item);
        }
        if let Some(found) = find_item_in_collections(&item.children, id) {
            return Some(found);
        }
    }
    None
}

/// 获取父集合的 children 数组可变引用
/// parent_id 为 None 时返回根级别 collections
pub fn find_parent_children<'a>(
    items: &'a mut Vec<Collection>,
    parent_id: Option<&str>,
) -> Option<&'a mut Vec<Collection>> {
    match parent_id {
        None => Some(items),
        Some(pid) => {
            for item in items.iter_mut() {
                if item.id == pid && item.item_type == "collection" {
                    return Some(&mut item.children);
                }
                if let Some(found) = find_parent_children(&mut item.children, Some(pid)) {
                    return Some(found);
                }
            }
            None
        }
    }
}

/// 递归获取集合深度
pub fn get_collection_depth(items: &[Collection], id: &str, current_depth: usize) -> Option<usize> {
    for item in items {
        if item.id == id {
            return Some(current_depth);
        }
        if let Some(d) = get_collection_depth(&item.children, id, current_depth + 1) {
            return Some(d);
        }
    }
    None
}

/// 获取集合的所有子孙 ID（用于检查是否移动到自己的子集）
pub fn get_all_descendant_ids(items: &[Collection], id: &str) -> Option<Vec<String>> {
    // 先找到该集合
    for item in items {
        if item.id == id && item.item_type == "collection" {
            // 收集所有子孙 ID
            let mut result = Vec::new();
            collect_descendant_ids(&item.children, &mut result);
            return Some(result);
        }
        if let Some(found) = get_all_descendant_ids(&item.children, id) {
            return Some(found);
        }
    }
    None
}

/// 递归收集子孙 ID
fn collect_descendant_ids(items: &[Collection], result: &mut Vec<String>) {
    for item in items {
        result.push(item.id.clone());
        collect_descendant_ids(&item.children, result);
    }
}

/// 获取集合的最大子层级深度（用于检查移动后是否超过层级限制）
pub fn get_collection_max_child_depth(items: &[Collection], id: &str) -> Option<usize> {
    // 先找到该集合
    for item in items {
        if item.id == id && item.item_type == "collection" {
            return Some(get_max_depth_in_tree(&item.children, 0));
        }
        if let Some(d) = get_collection_max_child_depth(&item.children, id) {
            return Some(d);
        }
    }
    None
}

/// 计算树的最大深度
fn get_max_depth_in_tree(items: &[Collection], current_depth: usize) -> usize {
    if items.is_empty() {
        return current_depth;
    }
    let mut max = current_depth;
    for item in items {
        let child_max = get_max_depth_in_tree(&item.children, current_depth + 1);
        if child_max > max {
            max = child_max;
        }
    }
    max
}

#[cfg(test)]
mod tests {
    use super::*;

    fn make_collection(id: &str, children: Vec<Collection>) -> Collection {
        // 复用领域服务的工厂方法构造默认实体，再覆盖 id 与 children，
        // 避免测试随 Collection 字段演进而频繁改动。
        let mut c = CollectionDomainService::create_collection_entity(id.to_string(), None);
        c.id = id.to_string();
        c.children = children;
        c
    }

    #[test]
    fn find_ancestor_chain_root_match() {
        let tree = vec![make_collection("a", vec![])];
        let mut path = Vec::new();
        assert!(find_ancestor_chain(&tree, "a", &mut path));
        assert_eq!(path.len(), 1);
        assert_eq!(path[0].id, "a");
    }

    #[test]
    fn find_ancestor_chain_nested_path() {
        // 树结构：a -> b -> c，查找 c 应返回完整祖先链 a, b, c
        let tree = vec![make_collection(
            "a",
            vec![make_collection("b", vec![make_collection("c", vec![])])],
        )];
        let mut path = Vec::new();
        assert!(find_ancestor_chain(&tree, "c", &mut path));
        let ids: Vec<_> = path.iter().map(|c| c.id.clone()).collect();
        assert_eq!(ids, vec!["a", "b", "c"]);
    }

    #[test]
    fn find_ancestor_chain_not_found_returns_false() {
        let tree = vec![make_collection("a", vec![])];
        let mut path = Vec::new();
        assert!(!find_ancestor_chain(&tree, "missing", &mut path));
        assert!(path.is_empty());
    }
}
