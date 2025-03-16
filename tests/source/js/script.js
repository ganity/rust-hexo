// 页面加载完成后执行
document.addEventListener('DOMContentLoaded', function() {
    // 添加代码块复制按钮
    if (document.querySelector('pre code')) {
        addCopyButtons();
    }
    
    // 移动设备菜单切换
    setupMobileMenu();
});

// 为代码块添加复制按钮
function addCopyButtons() {
    // 获取所有代码块
    const codeBlocks = document.querySelectorAll('pre code');
    
    codeBlocks.forEach(function(codeBlock) {
        // 确保父元素有相对定位
        const preElement = codeBlock.parentNode;
        if (window.getComputedStyle(preElement).position === 'static') {
            preElement.style.position = 'relative';
        }
        
        // 创建复制按钮
        const copyButton = document.createElement('button');
        copyButton.className = 'copy-btn';
        copyButton.textContent = '复制';
        
        // 添加点击事件
        copyButton.addEventListener('click', function() {
            const code = codeBlock.textContent;
            
            // 使用 Clipboard API 复制代码
            navigator.clipboard.writeText(code).then(function() {
                // 复制成功
                copyButton.textContent = '已复制!';
                
                // 2秒后恢复按钮文本
                setTimeout(function() {
                    copyButton.textContent = '复制';
                }, 2000);
            }).catch(function(err) {
                // 复制失败
                console.error('无法复制代码: ', err);
                copyButton.textContent = '复制失败';
                
                // 2秒后恢复按钮文本
                setTimeout(function() {
                    copyButton.textContent = '复制';
                }, 2000);
            });
        });
        
        // 将按钮添加到代码块
        preElement.appendChild(copyButton);
    });
}

// 设置移动设备菜单
function setupMobileMenu() {
    const navToggle = document.querySelector('.nav-toggle');
    
    if (navToggle) {
        navToggle.addEventListener('click', function() {
            const nav = document.querySelector('.site-nav');
            
            if (nav) {
                nav.classList.toggle('active');
            }
        });
    }
} 