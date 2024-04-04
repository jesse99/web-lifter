/* eslint-env es6 */
/* eslint no-undef: "off" */
/* eslint no-console: "warn" */
"use strict";

const TIMER_OFF = 0;
const TIMER_WAITING = 1;    // i.e. for duration exercise to finish
const TIMER_RESTING = 2;
const TIMER_MANUAL = 3;

let timer = TIMER_OFF;
let start_time = undefined;
let deadline = undefined;
let timer_id = undefined;
let reps = undefined;

let old_next_label = undefined; // for manual timer

function on_click_reps(event, title) {
    const dropdown = document.getElementById('reps_button');
    dropdown.innerText = title;
    reps = title.split(" ")[0];

    const items = document.querySelectorAll('.rep_item');
    items.forEach(item => {
        if (item.innerText == title) {
            item.classList.add('active');
        } else {
            item.classList.remove('active');
        }
    });
}

function update_clicked() {
    const update = document.getElementById('update_button');
    if (update.getAttribute("value") == "1") {
        update.setAttribute("value", "0");
    } else {
        update.setAttribute("value", "1");
    }
}

function advance_clicked() {
    const update = document.getElementById('advance_button');
    if (update.getAttribute("value") == "1") {
        update.setAttribute("value", "0");
    } else {
        update.setAttribute("value", "1");
    }
}

function on_next(event) {
    const body = document.getElementById('body');
    const wait = parseInt(body.getAttribute("data-wait"));

    if (timer == TIMER_OFF) {
        if (wait > 0) {
            // Start waiting
            let button = document.getElementById('next_button');
            button.innerHTML = "Stop Waiting";

            start_time = seconds();
            deadline = start_time + wait;
            timer = TIMER_WAITING;
            update_wait();
            timer_id = setInterval(on_timer, 1000); // ms
        } else {
            timer = TIMER_RESTING;
            start_resting();
        }
    } else if (timer == TIMER_WAITING) {
        // User said he is done waiting
        timer = TIMER_OFF;
        clearInterval(timer_id);
        timer_id = undefined;
        start_resting();
    } else if (timer == TIMER_RESTING) {
        // User said he is done resting
        clearInterval(timer_id);
        timer_id = undefined;
        post_next_set();
    } else {
        // User said he is done timing
        clearInterval(timer_id);
        timer = TIMER_OFF;
        timer_id = undefined;

        let label = document.getElementById('timer_text');
        label.innerHTML = "";

        let button = document.getElementById('next_button');
        button.innerHTML = old_next_label;
        old_next_label = undefined;
    }
}

// User requested to run a non-standard timer.
function start_manual_timer(event) {
    // TODO might be nice to override the current state but if we do that we'll need
    // to call post_next_set.
    if (timer == TIMER_OFF) {
        let button = document.getElementById('next_button');
        old_next_label = button.innerHTML;
        button.innerHTML = "Stop Timer";

        timer = TIMER_MANUAL;
        start_time = seconds();
        update_manual();
        timer_id = setInterval(on_timer, 1000); // ms
    }
}

function start_resting() {
    const body = document.getElementById('body');
    const rest = parseInt(body.getAttribute("data-rest"));
    if (rest > 0) {
        // Start resting
        const update = document.getElementById('reps_div');
        update.setAttribute("hidden", true);

        let button = document.getElementById('next_button');
        button.innerHTML = "Stop Resting";

        start_time = seconds();
        deadline = start_time + rest;
        update_rest();
        timer_id = setInterval(on_timer, 1000); // ms
    } else {
        // No need to rest so advance to next set
        post_next_set();
    }
}

// Note that we've told the browser to call us every second but that won't be perfectly
// reliable and errors will accumulate so we get the current time instead of relying on
// using a 1s interval.
function on_timer() {
    if (timer == TIMER_WAITING) {
        update_wait();
    } else if (timer == TIMER_RESTING) {
        update_rest();
    } else {
        update_manual();
    }
}

function update_wait() {
    const current = seconds();
    let label = document.getElementById('timer_text');
    const remaining = deadline - current;
    if (current < deadline) {
        label.innerHTML = friendly_time(remaining);
        label.style.color = "blue";
    } else {
        timer = TIMER_OFF;
        clearInterval(timer_id);
        timer_id = undefined;
        start_resting();
    }
}

function update_rest() {
    const current = seconds();
    let label = document.getElementById('timer_text');

    const remaining = deadline - current;
    if (current < deadline) {
        // console.log(`remaining: ${remaining}`);
        label.innerHTML = friendly_time(remaining);
        label.style.color = "red";
    } else if (current < deadline + 2) {
        label.innerHTML = "Done";
        label.style.color = "green";
    } else {
        label.innerHTML = friendly_time(-remaining) + " over";
        label.style.color = "green";
    }
}

function update_manual() {
    const current = seconds();
    let label = document.getElementById('timer_text');

    let elapsed = current - start_time;
    label.innerHTML = friendly_time(elapsed);
    label.style.color = "green";
}


function friendly_time(secs) {
    if (secs > 360) {
        return add_unit((secs / 360).toLocaleString(undefined, { maximumFractionDigits: 2 }), "hour");
    } else if (secs > 60) {
        return add_unit((secs / 60).toLocaleString(undefined, { maximumFractionDigits: 1 }), "min");
    } else {
        return add_unit(secs.toLocaleString(undefined, { maximumFractionDigits: 0 }), "sec");
    }
}

function add_unit(num, unit) {
    if (num == "1") {
        return num + " " + unit;
    } else {
        return num + " " + unit + "s";

    }
}

function format_int(x) {
    return x.toLocaleString(undefined, { maximumFractionDigits: 0 });
}

// since midnight, 1 Jan 1970
function seconds() {
    return new Date().getTime() / 1000;
}

// We can send a POST with XMLHttpRequest but that won't load a new page so what we do
// is dynamically create a form and submit that. This is based on 
// https://stackoverflow.com/questions/17378619/navigate-to-another-page-with-post-request-through-link.
function post_next_set() {
    const body = document.getElementById('body');
    const workout = body.getAttribute("data-workout");
    const exercise = body.getAttribute("data-exercise");

    var form = document.createElement('form');
    form.style.visibility = 'hidden'; // no user interaction is necessary
    form.method = 'POST'; // forms by default use GET query strings

    if (reps === undefined) {
        const dropdown = document.getElementById('reps_button');
        if (dropdown.innerText) {
            // user didn't change default
            reps = dropdown.innerText.split(" ")[0];
        }
    }
    if (reps !== undefined) {
        form.action = `/exercise/${workout}/${exercise}/next-var-set`;  // TODO escape this?
        form.action += `?reps=${reps}`;

        const update = document.getElementById('update_button');
        form.action += `&update=${update.getAttribute("value")}`;

        const advance = document.getElementById('advance_button');
        form.action += `&advance=${advance.getAttribute("value")}`;
    } else {
        form.action = `/exercise/${workout}/${exercise}/next-set`;
    }

    document.body.appendChild(form); // forms cannot be submitted outside of body
    form.submit(); // send the payload and navigate
}

function post_reset() {
    const body = document.getElementById('body');
    const workout = body.getAttribute("data-workout");
    const exercise = body.getAttribute("data-exercise");

    var form = document.createElement('form');
    form.style.visibility = 'hidden'; // no user interaction is necessary
    form.method = 'POST'; // forms by default use GET query strings
    form.action = `/reset/exercise/${workout}/${exercise}`;

    document.body.appendChild(form); // forms cannot be submitted outside of body
    form.submit(); // send the payload and navigate
}

let old_units = "mins";

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

function on_units_change(event) {
    let secs = event.target.children[0];
    let mins = event.target.children[1];
    let hours = event.target.children[2];
    if (secs.selected) {
        convert_time("rest-btn", "secs", 0)
        convert_time("last-rest-btn", "secs", 0)
        old_units = "secs";

    } else if (mins.selected) {
        convert_time("rest-btn", "mins", 2)
        convert_time("last-rest-btn", "mins", 2)
        old_units = "mins";

    } else if (hours.selected) {
        convert_time("rest-btn", "hours", 4)
        convert_time("last-rest-btn", "hours", 4)
        old_units = "hours";
    }
}

function on_loaded_rest() {
    // There's mention of a changed.bs.select event but I think that's not used with
    // Bootstrap 5.0's select widget.
    const units = document.getElementById('units');
    units.addEventListener("change", on_units_change, null);
}