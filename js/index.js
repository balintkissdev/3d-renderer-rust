import "../site/styles.css"

const spinner = document.getElementById("spinner");

import("../pkg")
    .catch((err) => {
        // winit crate throws an exception even on success:
        //
        // https://users.rust-lang.org/t/getting-rid-of-this-error-error-using-exceptions-for-control-flow-dont-mind-me-this-isnt-actually-an-error/92209/2
        // https://github.com/rust-windowing/winit/blob/2486f0f1a1d00ac9e5936a5222b2cfe90ceeca02/src/platform_impl/web/event_loop/mod.rs#L40
        //
        // Ignore it.
        if (!err.message.includes("Using exceptions for control flow, don't mind me. This isn't actually an error!")) {
            const errorMsg = `Failed to load WebAssembly module: ${err}`;
            console.error(errorMsg);
            alert(errorMsg);
        }
    })
    .finally(() => {
        spinner.remove();
    });
