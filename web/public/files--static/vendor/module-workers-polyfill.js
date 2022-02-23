(function(W) {
  if (W && W._$P === true) return;

  // if Worker is present and doesn't support Module Workers, install polyfill:
  if (W) {
    var s;
    var OPTS = Object.defineProperty({}, 'type', {
      get: function() {
        s = true;
      }
    });
    try {
      var url = URL.createObjectURL(new Blob([''],{type:'text/javascript'}));
      new W(url, OPTS).terminate();
      URL.revokeObjectURL(url);
    } catch (e) {}
    if (!s) {
      try {
        new W("data:text/javascript,", OPTS).terminate();
      } catch (e) {}
    }
    if (s) return;

    (self.Worker = function Worker(url, opts) {
      if (opts && opts.type == "module") {
        opts = { name: url + '\n' + (opts.name || '') };
        url = typeof document == "undefined" ? location.href : (document.currentScript && document.currentScript.src) || new Error().stack.match(/[(@]((file|https?):\/\/[^)]+?):\d+(:\d+)?(?:\)|$)/m)[1];
      }
      return new W(url, opts);
    })._$P = true;
  }

  function p() {
    // esm-polyfill
    var r = {}, n = {}, x;
    function e(r, e) {
      for (
        e = e.replace(/^(\.\.\/|\.\/)/, r.replace(/[^/]+$/g, "") + "$1");
        e !== (e = e.replace(/[^/]+\/\.\.\//g, ""));
      );
      return e.replace(/\.\//g, "");
    }
    function t(s, u) {
      var o, a = s;
      u && (s = e(u, s));
      return r[s] || (r[s] = fetch(s).then(function(u) {
        if ((a = u.url) !== s) {
          if (null != r[a]) return r[a];
          r[a] = r[s];
        }
        return u.text().then(function(r) {
          if (!u.ok) throw r;
          var m = { exports: {} };
          o = n[a] || (n[a] = m.exports);
          var i = function(r) { return t(r, a); }, c = [];
          return (
            (r = (function(r, e) {
              e = e || [];
              var n, t = [], j = 0;
              function u(r, e) {
                for (var s, u = /(?:^|,)\s*([\w$]+)(?:\s+as\s+([\w$]+))?\s*/g, o = []; s = u.exec(r); )
                  e ? t.push((s[2] || s[1]) + ":" + s[1]) : o.push((s[2] || s[1]) + "=" + n + "." + s[1]);
                return o;
              }
              return (
                r = r
                  .replace(
                    /(^\s*|[;}\s\n]\s*)import\s*(?:(?:([\w$]+)(?:\s*\,\s*\{([^}]+)\})?|(?:\*\s*as\s+([\w$]+))|\{([^}]*)\})\s*from)?\s*(['"])(.+?)\6/g,
                    function(r, t, o, a, b, i, c, p) {
                      return (
                        e.push(p),
                        (t += "var " + (n = "$im$" + ++j) + "=$require(" + c + p + c + ")"),
                        o && (t += ";var " + o + " = 'default' in " + n + " ? " + n + ".default : " + n),
                        b && (t += ";var " + b + " = " + n),
                        (a = a || i) && (t += ";var " + u(a, !1)),
                        t
                      );
                    }
                  )
                  .replace(
                    /((?:^|[;}\s\n])\s*)export\s*(?:\s+(default)\s+|((?:async\s+)?function\s*\*?|class|const\s|let\s|var\s)\s*([a-zA-Z0-9$_{[]+))/g,
                    function(r, e, n, u, o) {
                      if (n) {
                        var a = "$im$" + ++j;
                        return t.push("default:" + a), e + "var " + a + "=";
                      }
                      return t.push(o + ":" + o), e + u + " " + o;
                    }
                  )
                  .replace(
                    /((?:^|[;}\s\n])\s*)export\s*\{([^}]+)\}\s*;?/g,
                    function(r, e, n) {
                      return u(n, !0), e;
                    }
                  )
                  .replace(
                    /((?:^|[^a-zA-Z0-9$_@`'".])\s*)(import\s*\([\s\S]+?\))/g,
                    "$1$$$2"
                  )
              ).replace(
                /((?:^|[^a-zA-Z0-9$_@`'".])\s*)import\.meta\.url/g,
                "$1" + JSON.stringify(s)
              ) + "\n$module.exports={" + t.join(",") + "}";
            })(r, c)),
            Promise.all(
              c.map(function(r) {
                var s = e(a, r);
                return s in n ? n[s] : t(s);
              })
            ).then(function(e) {
              r += '\n//# sourceURL=' + s;
              try {
                var f = new Function("$import", "$require", "$module", "$exports", r);
              } catch (e) {
                var line = e.line - 1;
                var column = e.column;
                var lines = r.split('\n');
                var stack = (lines[line-2] || '') + '\n' + lines[line-1] + '\n' + (column==null?'':new Array(column).join('-')+'^\n') + (lines[line] || '');
                var err = new Error(e.message + '\n\n' + stack, s, line);
                err.sourceURL = err.fileName = s;
                err.line = line;
                err.column = column;
                throw err;
              }
              var n = f(i, function(r) { return e[c.indexOf(r)]; }, m, m.exports);
              return (
                null != n && (m.exports = n),
                Object.assign(o, m.exports),
                m.exports
              );
            })
          );
        });
      }));
    }

    // import while queuing messages
    var q = [], m = q.push.bind(q);
    addEventListener("message", m);
    function d() {
      removeEventListener("message", m);
      q.map(dispatchEvent);
    }
    var u = self.name.match(/^[^\n]+/)[0];
    self.name = self.name.replace(/^[^\n]*\n/g, '');
    t(u).then(d).catch(function(e) {
      setTimeout(function() {
        throw e;
      });
    });
  }

  if (typeof document == "undefined") p();
})(self.Worker);
