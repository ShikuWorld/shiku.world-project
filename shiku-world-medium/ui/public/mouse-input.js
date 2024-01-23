"use strict";
(() => {
  // node_modules/ts-pattern/dist/index.js
  var t = Symbol.for("@ts-pattern/matcher");
  var e = Symbol.for("@ts-pattern/isVariadic");
  var n = "@ts-pattern/anonymous-select-key";
  var r = (t2) => Boolean(t2 && "object" == typeof t2);
  var i = (e2) => e2 && !!e2[t];
  var s = (n2, o2, c2) => {
    if (i(n2)) {
      const e2 = n2[t](), { matched: r2, selections: i2 } = e2.match(o2);
      return r2 && i2 && Object.keys(i2).forEach((t2) => c2(t2, i2[t2])), r2;
    }
    if (r(n2)) {
      if (!r(o2))
        return false;
      if (Array.isArray(n2)) {
        if (!Array.isArray(o2))
          return false;
        let t2 = [], r2 = [], a = [];
        for (const s2 of n2.keys()) {
          const o3 = n2[s2];
          i(o3) && o3[e] ? a.push(o3) : a.length ? r2.push(o3) : t2.push(o3);
        }
        if (a.length) {
          if (a.length > 1)
            throw new Error("Pattern error: Using `...P.array(...)` several times in a single pattern is not allowed.");
          if (o2.length < t2.length + r2.length)
            return false;
          const e2 = o2.slice(0, t2.length), n3 = 0 === r2.length ? [] : o2.slice(-r2.length), i2 = o2.slice(t2.length, 0 === r2.length ? Infinity : -r2.length);
          return t2.every((t3, n4) => s(t3, e2[n4], c2)) && r2.every((t3, e3) => s(t3, n3[e3], c2)) && (0 === a.length || s(a[0], i2, c2));
        }
        return n2.length === o2.length && n2.every((t3, e2) => s(t3, o2[e2], c2));
      }
      return Object.keys(n2).every((e2) => {
        const r2 = n2[e2];
        return (e2 in o2 || i(a = r2) && "optional" === a[t]().matcherType) && s(r2, o2[e2], c2);
        var a;
      });
    }
    return Object.is(o2, n2);
  };
  var o = (e2) => {
    var n2, s2, a;
    return r(e2) ? i(e2) ? null != (n2 = null == (s2 = (a = e2[t]()).getSelectionKeys) ? void 0 : s2.call(a)) ? n2 : [] : Array.isArray(e2) ? c(e2, o) : c(Object.values(e2), o) : [];
  };
  var c = (t2, e2) => t2.reduce((t3, n2) => t3.concat(e2(n2)), []);
  function u(t2) {
    return Object.assign(t2, { optional: () => l(t2), and: (e2) => m(t2, e2), or: (e2) => y(t2, e2), select: (e2) => void 0 === e2 ? p(t2) : p(e2, t2) });
  }
  function l(e2) {
    return u({ [t]: () => ({ match: (t2) => {
      let n2 = {};
      const r2 = (t3, e3) => {
        n2[t3] = e3;
      };
      return void 0 === t2 ? (o(e2).forEach((t3) => r2(t3, void 0)), { matched: true, selections: n2 }) : { matched: s(e2, t2, r2), selections: n2 };
    }, getSelectionKeys: () => o(e2), matcherType: "optional" }) });
  }
  function m(...e2) {
    return u({ [t]: () => ({ match: (t2) => {
      let n2 = {};
      const r2 = (t3, e3) => {
        n2[t3] = e3;
      };
      return { matched: e2.every((e3) => s(e3, t2, r2)), selections: n2 };
    }, getSelectionKeys: () => c(e2, o), matcherType: "and" }) });
  }
  function y(...e2) {
    return u({ [t]: () => ({ match: (t2) => {
      let n2 = {};
      const r2 = (t3, e3) => {
        n2[t3] = e3;
      };
      return c(e2, o).forEach((t3) => r2(t3, void 0)), { matched: e2.some((e3) => s(e3, t2, r2)), selections: n2 };
    }, getSelectionKeys: () => c(e2, o), matcherType: "or" }) });
  }
  function d(e2) {
    return { [t]: () => ({ match: (t2) => ({ matched: Boolean(e2(t2)) }) }) };
  }
  function p(...e2) {
    const r2 = "string" == typeof e2[0] ? e2[0] : void 0, i2 = 2 === e2.length ? e2[1] : "string" == typeof e2[0] ? void 0 : e2[0];
    return u({ [t]: () => ({ match: (t2) => {
      let e3 = { [null != r2 ? r2 : n]: t2 };
      return { matched: void 0 === i2 || s(i2, t2, (t3, n2) => {
        e3[t3] = n2;
      }), selections: e3 };
    }, getSelectionKeys: () => [null != r2 ? r2 : n].concat(void 0 === i2 ? [] : o(i2)) }) });
  }
  function v(t2) {
    return "number" == typeof t2;
  }
  function b(t2) {
    return "string" == typeof t2;
  }
  function w(t2) {
    return "bigint" == typeof t2;
  }
  var S = u(d(function(t2) {
    return true;
  }));
  var j = (t2) => Object.assign(u(t2), { startsWith: (e2) => {
    return j(m(t2, (n2 = e2, d((t3) => b(t3) && t3.startsWith(n2)))));
    var n2;
  }, endsWith: (e2) => {
    return j(m(t2, (n2 = e2, d((t3) => b(t3) && t3.endsWith(n2)))));
    var n2;
  }, minLength: (e2) => j(m(t2, ((t3) => d((e3) => b(e3) && e3.length >= t3))(e2))), maxLength: (e2) => j(m(t2, ((t3) => d((e3) => b(e3) && e3.length <= t3))(e2))), includes: (e2) => {
    return j(m(t2, (n2 = e2, d((t3) => b(t3) && t3.includes(n2)))));
    var n2;
  }, regex: (e2) => {
    return j(m(t2, (n2 = e2, d((t3) => b(t3) && Boolean(t3.match(n2))))));
    var n2;
  } });
  var E = j(d(b));
  var K = (t2) => Object.assign(u(t2), { between: (e2, n2) => K(m(t2, ((t3, e3) => d((n3) => v(n3) && t3 <= n3 && e3 >= n3))(e2, n2))), lt: (e2) => K(m(t2, ((t3) => d((e3) => v(e3) && e3 < t3))(e2))), gt: (e2) => K(m(t2, ((t3) => d((e3) => v(e3) && e3 > t3))(e2))), lte: (e2) => K(m(t2, ((t3) => d((e3) => v(e3) && e3 <= t3))(e2))), gte: (e2) => K(m(t2, ((t3) => d((e3) => v(e3) && e3 >= t3))(e2))), int: () => K(m(t2, d((t3) => v(t3) && Number.isInteger(t3)))), finite: () => K(m(t2, d((t3) => v(t3) && Number.isFinite(t3)))), positive: () => K(m(t2, d((t3) => v(t3) && t3 > 0))), negative: () => K(m(t2, d((t3) => v(t3) && t3 < 0))) });
  var A = K(d(v));
  var x = (t2) => Object.assign(u(t2), { between: (e2, n2) => x(m(t2, ((t3, e3) => d((n3) => w(n3) && t3 <= n3 && e3 >= n3))(e2, n2))), lt: (e2) => x(m(t2, ((t3) => d((e3) => w(e3) && e3 < t3))(e2))), gt: (e2) => x(m(t2, ((t3) => d((e3) => w(e3) && e3 > t3))(e2))), lte: (e2) => x(m(t2, ((t3) => d((e3) => w(e3) && e3 <= t3))(e2))), gte: (e2) => x(m(t2, ((t3) => d((e3) => w(e3) && e3 >= t3))(e2))), positive: () => x(m(t2, d((t3) => w(t3) && t3 > 0))), negative: () => x(m(t2, d((t3) => w(t3) && t3 < 0))) });
  var P = x(d(w));
  var T = u(d(function(t2) {
    return "boolean" == typeof t2;
  }));
  var k = u(d(function(t2) {
    return "symbol" == typeof t2;
  }));
  var B = u(d(function(t2) {
    return null == t2;
  }));
  var W = { matched: false, value: void 0 };
  function N(t2) {
    return new $(t2, W);
  }
  var $ = class _$ {
    constructor(t2, e2) {
      this.input = void 0, this.state = void 0, this.input = t2, this.state = e2;
    }
    with(...t2) {
      if (this.state.matched)
        return this;
      const e2 = t2[t2.length - 1], r2 = [t2[0]];
      let i2;
      3 === t2.length && "function" == typeof t2[1] ? i2 = t2[1] : t2.length > 2 && r2.push(...t2.slice(1, t2.length - 1));
      let o2 = false, c2 = {};
      const a = (t3, e3) => {
        o2 = true, c2[t3] = e3;
      }, u2 = !r2.some((t3) => s(t3, this.input, a)) || i2 && !Boolean(i2(this.input)) ? W : { matched: true, value: e2(o2 ? n in c2 ? c2[n] : c2 : this.input, this.input) };
      return new _$(this.input, u2);
    }
    when(t2, e2) {
      if (this.state.matched)
        return this;
      const n2 = Boolean(t2(this.input));
      return new _$(this.input, n2 ? { matched: true, value: e2(this.input, this.input) } : W);
    }
    otherwise(t2) {
      return this.state.matched ? this.state.value : t2(this.input);
    }
    exhaustive() {
      if (this.state.matched)
        return this.state.value;
      let t2;
      try {
        t2 = JSON.stringify(this.input);
      } catch (e2) {
        t2 = this.input;
      }
      throw new Error(`Pattern matching error: no pattern matches value ${t2}`);
    }
    run() {
      return this.exhaustive();
    }
    returnType() {
      return this;
    }
  };

  // plugins/mouse-input.ts
  var plugin_id = "MOUSE";
  var activate_plugin$;
  var mouse_active = false;
  var plugin = {
    initialize: mouse_input,
    id: plugin_id,
    plugin_options: { mouse_mode: "PurelyDirectionalNoJump" }
  };
  function mouse_input(guest_input, activate_plugin) {
    activate_plugin$ = activate_plugin;
    activate_plugin$.subscribe((input_plugin_id) => {
      if (mouse_active && input_plugin_id !== plugin_id) {
        set_button(guest_input, "Left" /* Left */, false);
        set_button(guest_input, "Right" /* Right */, false);
        set_button(guest_input, "Up" /* Up */, false);
        set_button(guest_input, "Down" /* Down */, false);
        set_button(guest_input, "Jump" /* Jump */, false);
      }
      mouse_active = input_plugin_id === plugin_id;
    });
    document.addEventListener("mousemove", (mouse_event) => {
      if (mouse_active) {
        update_mouse_input_button(mouse_event, guest_input);
      }
    });
    document.addEventListener(
      "visibilitychange",
      function() {
        if (document.hidden) {
          set_button(guest_input, "Left" /* Left */, false);
          set_button(guest_input, "Right" /* Right */, false);
          set_button(guest_input, "Up" /* Up */, false);
          set_button(guest_input, "Down" /* Down */, false);
          set_button(guest_input, "Jump" /* Jump */, false);
        }
      },
      false
    );
    document.addEventListener("mouseleave", function(event) {
      if (event.clientY <= 0 || event.clientX <= 0 || event.clientX >= window.innerWidth || event.clientY >= window.innerHeight) {
        set_button(guest_input, "Left" /* Left */, false);
        set_button(guest_input, "Right" /* Right */, false);
        set_button(guest_input, "Up" /* Up */, false);
        set_button(guest_input, "Down" /* Down */, false);
        set_button(guest_input, "Jump" /* Jump */, false);
      }
    });
  }
  function update_mouse_input_button(mouse_event, guest_input) {
    const canvas = document.getElementById("canvas");
    if (!canvas) {
      return;
    }
    const width = canvas.offsetWidth / 2;
    const height = canvas.offsetHeight / 2;
    const cursor_x = mouse_event.x - width;
    const cursor_y = mouse_event.y - height;
    N(plugin.plugin_options.mouse_mode).with("UpIsJumpAndNoDown", () => {
      set_button(guest_input, "Jump" /* Jump */, cursor_y < -50);
      left_right_standard(cursor_x, guest_input);
    }).with("PurelyDirectionalNoJump", () => {
      up_down_standard(cursor_y, guest_input);
      left_right_standard(cursor_x, guest_input);
    }).exhaustive();
  }
  function left_right_standard(cursor_x, guest_input) {
    if (cursor_x < -50) {
      set_button(guest_input, "Left" /* Left */, true);
      set_button(guest_input, "Right" /* Right */, false);
    }
    if (cursor_x >= -50 && cursor_x <= 50) {
      set_button(guest_input, "Left" /* Left */, false);
      set_button(guest_input, "Right" /* Right */, false);
    }
    if (cursor_x > 50) {
      set_button(guest_input, "Right" /* Right */, true);
      set_button(guest_input, "Left" /* Left */, false);
    }
  }
  function up_down_standard(cursor_y, guest_input) {
    if (cursor_y < -50) {
      set_button(guest_input, "Up" /* Up */, true);
      set_button(guest_input, "Down" /* Down */, false);
    }
    if (cursor_y >= -50 && cursor_y <= 50) {
      set_button(guest_input, "Up" /* Up */, false);
      set_button(guest_input, "Down" /* Down */, false);
    }
    if (cursor_y > 50) {
      set_button(guest_input, "Up" /* Up */, false);
      set_button(guest_input, "Down" /* Down */, true);
    }
  }
  function set_button(guest_input, button, button_state) {
    if (guest_input.button_pressed_map[button] !== button_state) {
      guest_input.button_pressed_map[button] = button_state;
      guest_input.is_dirty = true;
    }
  }
  if (window.register_input_plugin) {
    window.register_input_plugin(plugin);
  }
})();
//# sourceMappingURL=mouse-input.js.map
