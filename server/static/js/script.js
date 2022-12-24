function on_submit(form) {
    window.location.href = "/waiting.html"
    return true
}

function reset_forms() {
    Array.from(document.forms).map(form => form.reset())
}