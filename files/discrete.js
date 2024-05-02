/* eslint-env es6 */
/* eslint no-undef: "off" */
/* eslint no-console: "warn" */
"use strict";

function on_loaded() {
    update_value();
    enable_menu();

    // autofocus attribute doesn't work within modals so we need to do it manually
    let modal = document.getElementById("add_modal");
    let weight = document.getElementById("weight-input");
    modal.addEventListener('shown.bs.modal', () => {
        weight.focus()
    })
}

function get_values() {
    let values = [];

    const list = document.getElementById('list');
    const len = list.children.length;
    for (let i = 0; i < len; i++) {
        let child = list.children[i];
        let parts = child.innerText.split(" ");
        values.push(parseFloat(parts[0]));
    }

    return values;
}

function has_value(value) {
    let values = get_values();

    for (const candidate of values) {
        if (Math.abs(candidate - value) < 0.01) {
            return true;
        }
    }

    return false;
}

function resort() {
    const list = document.getElementById('list');
    [...list.children]
        .sort((a, b) => {
            const x = parseFloat(a.innerText.split(" ")[0]);
            const y = parseFloat(b.innerText.split(" ")[0]);
            return x > y ? 1 : -1
        })
        .forEach(node => list.appendChild(node));
}

// TODO would be great to validate the input within the modal
// 1) dismiss the modal only if it validates
// 2) provide some sort of indication that it's invalid, popup? error text?
// 3) be sure to catch empty (or all whitespace) too
// https://stackoverflow.com/questions/27968361/validate-input-text-in-bootstrap-modal
function on_add(id) {
    let weight = document.getElementById(id);
    if (weight.value) {
        let parts = weight.value.split(" ");
        let value = parseFloat(parts[0]);
        if (value > 0.0 && !has_value(value)) {
            let item = document.createElement("li");
            item.classList.add("list-group-item");
            item.setAttribute("onclick", "on_click(this)");
            item.innerText = `${weight.value} lbs`;

            const list = document.getElementById('list');
            list.appendChild(item);

            resort();
            update_value();
            enable_menu();
        }
    }
}

function on_delete() {
    const list = document.getElementById('list');
    const len = list.children.length;
    for (let i = 0; i < len; i++) {
        let child = list.children[i];
        if (child.classList.contains('active')) {
            list.removeChild(child);
            update_value();
            enable_menu();
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
    enable_menu();
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

function enable_menu() {
    let button = document.getElementById("delete-btn");
    button.classList.add('disabled');

    const list = document.getElementById('list');
    const len = list.children.length;
    for (let i = 0; i < len; i++) {
        let child = list.children[i];
        if (child.classList.contains('active')) {
            button.classList.remove('disabled');
            break;
        }
    }
}

window.addEventListener('DOMContentLoaded', on_loaded);
