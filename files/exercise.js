/* eslint-env es6 */
/* eslint no-undef: "off" */
/* eslint no-console: "warn" */
"use strict";

let waiting = undefined;
let start_time = undefined;
let deadline = undefined;
let timer_id = undefined;
let reps = undefined;

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

    if (deadline === undefined) {
        if (wait > 0) {
            // Start waiting
            let button = document.getElementById('next_button');
            button.innerHTML = "Stop Waiting";

            start_time = seconds();
            deadline = start_time + wait;
            waiting = true;
            update_wait();
            timer_id = setInterval(on_timer, 1000); // ms
        } else {
            waiting = false;
            start_resting();
        }
    } else if (waiting) {
        // User said he is done waiting
        waiting = false;
        clearInterval(timer_id);
        timer_id = undefined;
        start_resting();
    } else {
        // User said he is done resting
        clearInterval(timer_id);
        timer_id = undefined;
        post_next_set();
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

function update_wait() {
    const current = seconds();
    let label = document.getElementById('timer_text');
    const remaining = deadline - current;
    if (current < deadline) {
        label.innerHTML = friendly_time(remaining);
        label.style.color = "blue";
    } else {
        waiting = false;
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
        label.innerHTML = "+" + friendly_time(-remaining);
        label.style.color = "green";
    }
}

// Note that we've told the browser to call us every second but that won't be perfectly
// reliable and errors will accumulate so we get the current time instead of relying on
// using a 1s interval.
function on_timer() {
    if (waiting) {
        update_wait();
    } else {
        update_rest();
    }
}

function friendly_time(secs) {
    if (secs > 360) {
        return (secs / 360).toLocaleString(undefined, { maximumFractionDigits: 2 }) + " hours";
    } else if (secs > 60) {
        return (secs / 60).toLocaleString(undefined, { maximumFractionDigits: 1 }) + " mins";
    } else {
        return secs.toLocaleString(undefined, { maximumFractionDigits: 0 }) + " secs";
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
        form.action = `/exercise/${workout}/${exercise}/next-var-set`;
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
