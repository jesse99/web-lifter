/* eslint-env es6 */
/* eslint no-undef: "off" */
/* eslint no-console: "warn" */
"use strict";

function on_loaded() {
    update_value();
}

function on_add() {
    update_value();
}

function on_delete() {
    update_value();
}

// TODO need to disable delete menu item if no selection
function on_click(item) {
    const list = document.getElementById('list');
    for (var child of list.children) {
        child.classList.remove('active');
        child.setAttribute('aria-current', "false");
    }
    item.classList.add('active');
    item.setAttribute('aria-current', "true");
}

function update_value() {
    let input = document.getElementById('list-button');
    input.value = "";

    const list = document.getElementById('list');
    for (var child of list.children) {
        input.value += child.innerText;
        input.value += " ";
    }
}

window.addEventListener('DOMContentLoaded', on_loaded);
