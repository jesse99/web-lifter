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

function parse_values() {
    let input = document.getElementById("weight-input");
    if (input.value) {
        // 5-100 by 10
        let parts = input.value.trim().split(/\s+/);
        if (parts.length == 3 && parts[1] == "by") {
            let step = parseFloat(parts[2]);
            parts = parts[0].split("-");
            if (step && step > 0.0 && parts.length == 2) {
                let min = parseFloat(parts[0]);
                let max = parseFloat(parts[1]);
                if (min && max && min > 0.0 && min <= max) {
                    let weight = min;
                    let weights = [];
                    while (weight <= max) {
                        weights.push(weight);
                        weight += step;
                    }
                    return weights;
                }
            }
        } else if (parts.length == 1) {
            // 45
            let weight = parseFloat(parts[0]);
            if (weight > 0.0) {
                return [weight];
            }
        }
    }
    return undefined;
}

function on_save() {
    const weights = parse_values();

    let help = document.getElementById("weight-help");
    if (weights) {
        help.classList.remove("text-danger");

        let added = false;
        for (var weight of weights) {
            if (add_weight(weight)) {
                added = true;
            }
        }
        if (added) {
            resort();
            update_value();
            enable_menu();
        }

        let modal = document.getElementById("add_modal");
        bootstrap.Modal.getInstance(modal).hide();
    } else {
        help.classList.add("text-danger");
    }
}

function add_weight(weight) {
    if (!has_value(weight)) {
        let item = document.createElement("li");
        item.classList.add("list-group-item");
        item.setAttribute("onclick", "on_click(this)");
        item.innerText = `${weight} lbs`;

        const list = document.getElementById('list');
        list.appendChild(item);
        return true;
    }
    return false;
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
