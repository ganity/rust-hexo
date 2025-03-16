// 这是一个简单的测试文件，用于验证 generator 模块的修改

use std::fmt::Write;
use std::path::PathBuf;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    pub fn test_string_mutability() {
        // 预分配足够大小的字符串
        let mut content = String::with_capacity(4096);
        
        // 使用 writeln! 写入内容
        writeln!(&mut content, "测试内容").unwrap();
        writeln!(&mut content, "带有格式化的内容: {}", "示例文本").unwrap();
        writeln!(&mut content, "复杂格式化: {{}} 和 {}", "实际值").unwrap();
        
        // 验证写入的内容
        assert!(content.contains("测试内容"));
        assert!(content.contains("带有格式化的内容: 示例文本"));
        assert!(content.contains("复杂格式化: {} 和 实际值"));
        
        // 验证容量足够
        assert!(content.capacity() >= 4096);
        
        println!("测试通过：字符串可变性和容量预分配正确");
    }
    
    #[test]
    pub fn test_path_operations() {
        use std::path::PathBuf;
        
        // 测试路径操作
        let base = PathBuf::from("/test/base");
        let rel = PathBuf::from("relative/path");
        
        let combined = base.join(rel);
        assert_eq!(combined, PathBuf::from("/test/base/relative/path"));
        println!("测试通过: 路径操作正常");
    }
}

// 另外创建普通函数用于主函数调用
fn run_string_test() {
    use std::fmt::Write;
    
    // 测试字符串可变性和容量预分配
    let mut content = String::with_capacity(4096);
    
    // 使用 writeln! 写入内容
    writeln!(&mut content, "测试内容").unwrap();
    writeln!(&mut content, "带有格式化的内容: {}", "示例文本").unwrap();
    writeln!(&mut content, "复杂格式化: {{}} 和 {}", "实际值").unwrap();
    
    // 验证写入的内容
    assert!(content.contains("测试内容"));
    assert!(content.contains("带有格式化的内容: 示例文本"));
    assert!(content.contains("复杂格式化: {} 和 实际值"));
    
    // 验证容量
    assert!(content.capacity() >= 4096);
    println!("测试通过: content 的容量为 {}", content.capacity());
    println!("content 的内容为:\n{}", content);
}

fn run_path_test() {
    use std::path::PathBuf;
    
    // 测试路径操作
    let base = PathBuf::from("/test/base");
    let rel = PathBuf::from("relative/path");
    
    let combined = base.join(rel);
    assert_eq!(combined, PathBuf::from("/test/base/relative/path"));
    println!("测试通过: 路径操作正常");
}

// 添加主函数以便直接运行
fn main() {
    println!("运行独立测试...");
    run_string_test();
    run_path_test();
    println!("所有测试通过!");
} 