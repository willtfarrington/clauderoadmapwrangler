// Roadmap Wrangler's phone pages: the live updates (D-14, D-28; EP-30b). Every page works the same
// without this script. The window's one event stream names each part of its state that changed,
// by the path of the page that shows it. A page that listens to that path (its main element's
// data-listen) fetches itself again and puts in each part (data-part) that differs, leaving alone
// a part that holds the focused field or text typed and not sent, which it fetches again once the
// focus leaves. The stream is opened with the version of the window's state the page was drawn
// from, and names everything ("*") first when the window has moved on since: a change between the
// drawing and the opening, or while the window quit or restarted, which the browser opens the
// stream again after.
"use strict";
(function () {
  var main = document.querySelector("main[data-listen]");
  var listens = main ? main.getAttribute("data-listen").split(" ").filter(Boolean) : [];
  if (!listens.length || !window.EventSource || !window.DOMParser || !window.fetch) return;
  var version = main.getAttribute("data-version") || "";
  var fetching = false, again = false, held = false;

  // A part the owner is at: the focused field, or a field whose value or tick is not the page's.
  function typedIn(part) {
    if (part.contains(document.activeElement)) return true;
    var fields = part.querySelectorAll("input, textarea, select");
    for (var i = 0; i < fields.length; i++) {
      var field = fields[i];
      if (field.type === "hidden" || field.type === "submit") continue;
      if (field.type === "checkbox" || field.type === "radio") {
        if (field.checked !== field.defaultChecked) return true;
      } else if (field.value !== field.defaultValue) return true;
    }
    return false;
  }

  // The folds the owner opened or closed stay as they are where the part keeps its shape.
  function keepFolds(part, next) {
    var mine = part.querySelectorAll("details"), theirs = next.querySelectorAll("details");
    if (mine.length !== theirs.length) return;
    for (var i = 0; i < mine.length; i++) {
      if (mine[i].open) theirs[i].setAttribute("open", "");
      else theirs[i].removeAttribute("open");
    }
  }

  function refresh() {
    if (fetching) { again = true; return; }
    fetching = true;
    fetch(location.pathname, { credentials: "same-origin", cache: "no-store" })
      .then(function (answer) { return answer.ok ? answer.text() : null; })
      .then(function (text) {
        if (text === null) return;
        var fresh = new DOMParser().parseFromString(text, "text/html");
        var parts = document.querySelectorAll("[data-part]");
        for (var i = 0; i < parts.length; i++) {
          var part = parts[i], name = part.getAttribute("data-part");
          var next = fresh.querySelector('[data-part="' + name + '"]');
          if (!next) continue;
          keepFolds(part, next);
          if (next.innerHTML === part.innerHTML) continue;
          if (typedIn(part)) { held = true; continue; }
          part.replaceWith(document.importNode(next, true));
        }
        var asOf = fresh.querySelector(".as-of"), here = document.querySelector(".as-of");
        if (asOf && here) here.textContent = asOf.textContent;
      })
      .catch(function () {})
      .then(function () {
        fetching = false;
        if (again) { again = false; refresh(); }
      });
  }

  // The browser opens the stream again after a dropped connection, but never after an answer
  // other than the stream, such as the proxy's while the window is down: then it is opened anew,
  // a little later each time, with the page's version, which names everything once it is back.
  var since = /^[0-9]+$/.test(version) ? "?since=" + version : "";
  var wait = 5000;
  function listen() {
    var events = new EventSource("/m/events" + since);
    events.addEventListener("open", function () { wait = 5000; });
    events.addEventListener("change", function (event) {
      if (event.data === "*" || listens.indexOf(event.data) >= 0) refresh();
    });
    events.addEventListener("error", function () {
      if (events.readyState !== EventSource.CLOSED) return;
      setTimeout(listen, wait);
      wait = Math.min(wait * 2, 60000);
    });
  }
  listen();
  document.addEventListener("focusout", function () {
    if (held) { held = false; setTimeout(refresh, 300); }
  });
})();
