/* eslint-env es6 */
/* eslint no-undef: "off" */
/* eslint no-console: "warn" */
"use strict";

function on_loaded() {
    update_value();

    const list = document.getElementById('list');
    for (var child of list.children) {
        if (child.classList.contains('active')) {
            let text = child.getAttribute("data-help");
            if (text) {
                let help = document.getElementById('list-help');
                help.innerText = text;
            }
            break;
        }
    }
}

function on_click(item) {
    const list = document.getElementById('list');
    for (var child of list.children) {
        child.classList.remove('active');
        child.setAttribute('aria-current', "false");
    }
    item.classList.add('active');
    item.setAttribute('aria-current', "true");
    update_value();

    let text = item.getAttribute("data-help");
    if (text) {
        let help = document.getElementById('list-help');
        help.innerText = text;
    }
}

function update_value() {
    let input = document.getElementById('list-button');
    input.value = "";

    const list = document.getElementById('list');
    for (var child of list.children) {
        if (child.classList.contains("active")) {
            input.value = child.innerText;
            break;
        }
    }
}

window.addEventListener('DOMContentLoaded', on_loaded);
