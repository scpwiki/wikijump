dojo.provide("dojox.lang.tests.mix");

dojo.require("dojox.lang.functional.object");
dojo.require("dojox.lang.mix");

(function(){
	var df = dojox.lang.functional, mix = dojox.lang.mix,
		x = {a: 1, b: 2, c: 3}, y = {c: 1, d: 2, e: 3, f: 4},
		z = mix.cloneProps(y, {d: "a", e: "b", f: "q"}, ["f"]),
		q = dojo.clone(x), p = dojo.clone(y),
		print = function(v, i){ this.push("[" + i + "] = " + v); },
		show = function(o){ return df.forIn(o, print, []).sort().join(", "); };
		
	mix.copyProps(q, y);
	mix.copyProps(p, x);
	mix.processProps(y, {d: "a", e: "b", f: "q"}, ["f"]);
	
	tests.register("dojox.lang.tests.mix", [
		function testCopyProps(t){ t.assertEqual(df.keys(q).sort(), df.keys(p).sort()); },
		function testCloneProps(t){ t.assertEqual(df.keys(x).sort(), df.keys(z).sort()); },
		function testProcessProps(t){ t.assertEqual(show(y), show(z)); }
	]);
})();
