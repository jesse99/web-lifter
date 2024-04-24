/* eslint-env es6 */
/* eslint no-undef: "off" */
/* eslint no-console: "warn" */
"use strict";

let old_units = "mins";

function on_loaded() {
    const units = document.getElementById('units-select');
    units.addEventListener("change", on_units_change, null);
}

function on_units_change(event) {
    let secs = event.target.children[0];
    let mins = event.target.children[1];
    let hours = event.target.children[2];
    if (secs.selected) {
        convert_times("times-input", "secs", 0)
        convert_time("target-input", "secs", 0)
        old_units = "secs";

    } else if (mins.selected) {
        convert_times("times-input", "mins", 2)
        convert_time("target-input", "mins", 2)
        old_units = "mins";

    } else if (hours.selected) {
        convert_times("times-input", "hours", 4)
        convert_time("target-input", "hours", 4)
        old_units = "hours";
    }
}

function convert_times(name, new_units, decimals) {
    const input = document.getElementById(name);
    if (input.value && input.value.trim().length) {
        let new_values = [];
        for (const value of input.value.split(/\s+/)) {
            if (value) {
                new_values.push(convert(value, new_units, decimals));
            }
        }
        input.value = new_values.join(" ");
    }
}

function convert_time(name, new_units, decimals) {
    const input = document.getElementById(name);
    if (input && input.value && input.value.trim().length) {
        input.value = convert(input.value, new_units, decimals);
    }
}

function convert(value, new_units, decimals) {
    let time = parseFloat(value);
    if (old_units == "mins") {
        time = 60.0 * time;
    } else if (old_units == "hours") {
        time = 60.0 * 60.0 * time;
    }

    if (new_units == "mins") {
        time = time / 60.0;
    } else if (new_units == "hours") {
        time = time / (60.0 * 60.0);
    }

    return time.toFixed(decimals);
}

window.addEventListener('DOMContentLoaded', on_loaded);
