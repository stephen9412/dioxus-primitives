# Dioxus RSX 語法與UI描述

Dioxus 使用名為 RSX (類似於 React JSX) 的聲明式語法來描述用戶界面。這種語法允許開發者直觀地定義UI元素、屬性和結構。

## 1. `rsx!` 宏基礎

`rsx!` 宏是 Dioxus 中構建虛擬 DOM 的核心機制。它允許開發者使用類似 HTML 的語法直接在 Rust 代碼中描述 UI。

```rust
// 基本語法示例
rsx! {
    div {
        class: "container",
        h1 { "標題" }
        p { "這是段落內容" }
    }
}
```

## 2. 元素與屬性

### 2.1 基本元素

RSX 支援所有標準 HTML 元素，如 `div`、`span`、`h1`、`p` 等：

```rust
rsx! {
    div {
        h1 { "大標題" }
        p { "段落內容" }
        button { "點擊我" }
    }
}
```

### 2.2 屬性設置

元素屬性使用 `name: value` 語法定義：

```rust
rsx! {
    img {
        src: "https://example.com/image.png",
        alt: "示例圖片",
        width: "500px",
        height: "300px",
    }
}
```

#### 保留關鍵字處理

對於與 Rust 關鍵字衝突的屬性（如 `type`），使用 `r#` 前綴：

```rust
rsx! {
    input {
        r#type: "text",
        placeholder: "請輸入文字",
    }
}
```

#### 條件屬性

可以使用 if 語句有條件地添加屬性：

```rust
let is_active = true;
rsx! {
    div {
        class: if is_active { "active" },
        "當前狀態"
    }
}
```

#### 自定義屬性

可以通過引號包裹名稱來指定非標準屬性：

```rust
rsx! {
    div {
        "data-testid": "user-profile",
        "aria-label": "用戶資料區塊",
    }
}
```

### 2.3 特殊屬性

#### 危險內部 HTML (Dangerously Set Inner HTML)

當需要直接插入 HTML 字符串時，可以使用 `dangerous_inner_html` 屬性：

```rust
let html_content = "<b>加粗文字</b><i>斜體文字</i>";
rsx! {
    div {
        dangerous_inner_html: "{html_content}"
    }
}
```

> 注意：使用 `dangerous_inner_html` 時必須確保內容來源可信，否則可能導致 XSS 攻擊風險。

#### 布爾屬性

某些屬性（如 `hidden`、`disabled` 等）可以通過布爾值控制是否出現：

```rust
rsx! {
    input {
        disabled: form_locked,
        required: true,
    }
    div {
        hidden: should_hide,
        "此內容可能被隱藏"
    }
}
```

## 3. 字符串插值

RSX 支援在文本和屬性中進行字符串插值：

```rust
let name = "使用者";
let counter = 42;
rsx! {
    div {
        class: "user-{name}-container",
        h1 { "歡迎, {name}!" }
        p { "您有 {counter} 條未讀消息" }
    }
}
```

字符串格式化支援 Rust 標準格式說明符：

```rust
let coordinates = (10.5, 20.7);
rsx! {
    div {
        "位置: ({coordinates.0:.2}, {coordinates.1:.2})"
    }
}
```

## 4. 動態內容與控制流

### 4.1 表達式

可以在 RSX 中使用 `{}` 包裹 Rust 表達式：

```rust
let count = 5;
rsx! {
    div {
        {count * 2}
        {format!("計算結果: {}", count + 10)}
    }
}
```

### 4.2 條件渲染

使用 if/else 控制流：

```rust
let is_logged_in = true;
rsx! {
    div {
        if is_logged_in {
            h1 { "歡迎回來" }
            button { "登出" }
        } else {
            h1 { "請登入" }
            button { "登入" }
        }
    }
}
```

也可以單獨使用 if 而無需 else 分支：

```rust
rsx! {
    div {
        if show_header {
            h1 { "頁面標題" }
        }
    }
}
```

### 4.3 列表渲染

使用 for 循環或迭代器映射：

```rust
// 使用 for 循環
let items = vec!["蘋果", "香蕉", "橙子"];
rsx! {
    ul {
        for item in &items {
            li { "{item}" }
        }
    }
}

// 使用迭代器 map
rsx! {
    ul {
        {items.iter().map(|item| rsx! {
            li { "{item}" }
        })}
    }
}
```

## 5. 組件組合

RSX 允許在一個組件中使用其他組件：

```rust
fn Header() -> Element {
    rsx! {
        header {
            h1 { "網站標題" }
            nav { "導航內容..." }
        }
    }
}

fn App() -> Element {
    rsx! {
        div {
            Header {}
            main {
                "主要內容區域"
            }
            footer {
                "頁腳內容"
            }
        }
    }
}
```

## 6. 片段 (Fragments)

可以在 RSX 中返回多個頂層元素，它們會被自動分組：

```rust
rsx! {
    h1 { "標題" }
    p { "第一段" }
    p { "第二段" }
}
```

## 7. RSX 注意事項與最佳實踐

1. **格式化**: 保持一致的縮進和結構，提高可讀性
2. **key 屬性**: 渲染列表時，使用唯一的 key 屬性幫助 Dioxus 追蹤元素變化
3. **條件渲染**: 對於複雜條件，考慮在 RSX 外提前計算值
4. **避免過深嵌套**: 將複雜 UI 拆分為多個組件，而非創建深度嵌套結構
5. **屬性順序**: 保持一致的屬性順序（如 id、class、事件處理器、其他屬性）

以上是 Dioxus RSX 語法的核心特性，它提供了一種聲明式且直觀的方式來構建用戶界面，同時保持與 Rust 語言的無縫集成。
