/* eslint-env es6 */
/* eslint no-undef: "off" */
"use strict";

function on_loaded() {
    update_value();
    enable_menu();

    // autofocus attribute doesn't work within modals so we need to do it manually
    let modal = document.getElementById("add_modal");
    let weight = document.getElementById("weight-input");
    let count = document.getElementById("count-input");
    modal.addEventListener('shown.bs.modal', () => {
        weight.value = "";
        count.value = "";
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

function get_counts() {
    let values = [];

    const list = document.getElementById('list');
    const len = list.children.length;
    for (let i = 0; i < len; i++) {
        let child = list.children[i];
        let parts = child.innerText.split(" ");
        values.push(parseInt(parts[2].slice(1)));
    }

    return values;
}

function find_value(value) {
    let values = get_values();

    for (let i = 0; i < values.length; i++) {
        if (Math.abs(value - values[i]) < 0.01) {
            return i;
        }
    }

    return undefined;
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

function parse_value() {
    let input = document.getElementById("weight-input");
    if (input.value) {
        let weight = parseFloat(input.value);
        if (weight > 0.0) {
            return weight;
        }
    }
    return undefined;
}

function parse_count() {
    let input = document.getElementById("count-input");
    if (input.value) {
        let count = parseInt(input.value);
        if (count > 0) {
            return count;
        }
    }
    return undefined;
}

function on_save() {
    const weight = parse_value();
    const count = parse_count();

    let help = document.getElementById("weight-help");
    if (weight) {
        help.classList.remove("text-danger");
    } else {
        help.classList.add("text-danger");
    }

    help = document.getElementById("count-help");
    if (count) {
        help.classList.remove("text-danger");
    } else {
        help.classList.add("text-danger");
    }

    if (weight && count) {
        add_plate(weight, count);
        resort();
        update_value();
        enable_menu();

        let modal = document.getElementById("add_modal");
        bootstrap.Modal.getInstance(modal).hide();
    }
}

function add_plate(weight, count) {
    let i = find_value(weight);
    if (i !== undefined) {
        const list = document.getElementById('list');
        let child = list.children[i];
        list.removeChild(child);
    }

    let item = document.createElement("li");
    item.classList.add("list-group-item");
    item.setAttribute("onclick", "on_click(this)");
    item.innerText = `${weight} lbs x${count}`;

    const list = document.getElementById('list');
    list.appendChild(item);
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
    let values = get_values();
    let counts = get_counts();
    values = values.map((s) => s.toFixed(3));
    console.assert(values.length == counts.length);

    let items = [];
    for (let i = 0; i < values.length; i++) {
        items.push(`${values[i]}x${counts[i]}`)
    }

    let input = document.getElementById('list-button');
    input.value = items.join("¦");
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
