dojo.provide("dojox.lang.tests.array");

dojo.require("dojox.math");

(function(){
	tests.register("dojox.math.tests.factorial", [
		function fact0(t){ t.assertEqual(1, dojox.math.factorial(0)); },
		function fact5(t){ t.assertEqual(120, dojox.math.factorial(5)); },
		function factneg(t){ t.assertTrue(isNaN(dojox.math.factorial(-1))); }
	]);
})();
