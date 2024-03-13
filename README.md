yo-html
=======

JSX-like macro similar to what you can find in React or Yew but without
framework nor trait.

```rust
let name = "Tippsie";
let class = "class";
let onclick = todo!();
let dynamic_attribute = "style";
html! {
    <>      // Support fragments
        <div class="important">{"Hello "}<strong>{name}</strong></div>
        <ul class=["list", "of", class]>
            <li><button {onclick}>{"Click here"}</button></li>
            <li {dynamic_attribute}="color:red"></li>
            <li>("%x", 42)</li>     // Shorthand for: format_args!("%x", 42)
        </ul>
    </>
}
```

An example of web framework is provided in the `examples` directory but you
need to make your own for this macro to be usable.
