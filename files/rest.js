/* eslint-env es6 */
/* eslint no-undef: "off" */
/* eslint no-console: "warn" */
"use strict";

let old_units = "mins";

function on_loaded() {
    // There's mention of a changed.bs.select event but I think that's not used with
    // Bootstrap 5.0's select widget.
    const units = document.getElementById('units-select');
    units.addEventListener("change", on_units_change, null);
}

function on_units_change(event) {
    let secs = event.target.children[0];
    let mins = event.target.children[1];
    let hours = event.target.children[2];
    if (secs.selected) {
        convert_time("rest-input", "secs", 0)
        convert_time("last-rest-input", "secs", 0)
        old_units = "secs";

    } else if (mins.selected) {
        convert_time("rest-input", "mins", 2)
        convert_time("last-rest-input", "mins", 2)
        old_units = "mins";

    } else if (hours.selected) {
        convert_time("rest-input", "hours", 4)
        convert_time("last-rest-input", "hours", 4)
        old_units = "hours";
    }
}

function convert_time(name, new_units, decimals) {
    const input = document.getElementById(name);
    if (input.value && input.value.trim().length) {
        let time = parseFloat(input.value);
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

        input.value = time.toFixed(decimals);
    }
}

window.addEventListener('DOMContentLoaded', on_loaded);
