dojo.provide("dojox.json.tests.ref");
dojo.require("dojox.json.ref");

doh.register("dojox.json.tests.ref", [
	function fromRefJson(t) {
		var testStr = '{a:{$ref:"#"},id:"root",c:{d:"e",f:{$ref:"root.c"}},b:{$ref:"#.c"}}';

		var mirrorObj = dojox.json.ref.fromJson(testStr);
		t.assertEqual(mirrorObj, mirrorObj.a);
		t.assertEqual(mirrorObj.c, mirrorObj.c.f);
		t.assertEqual(mirrorObj.c, mirrorObj.b);
	},
	function toAndFromRefJson(t) {
		var testObj = {a:{},b:{c:{}}};
		testObj.a.d= testObj;
		testObj.b.g=testObj.a;
		testObj.b.c.f = testObj.b;
		testObj.b.h=testObj.a;
		var mirrorObj = dojox.json.ref.fromJson(dojox.json.ref.toJson(testObj));
		t.assertEqual(mirrorObj.a.d, mirrorObj);
		t.assertEqual(mirrorObj.b.g, mirrorObj.a);
		t.assertEqual(mirrorObj.b.c.f, mirrorObj.b);
		t.assertEqual(mirrorObj.b.h, mirrorObj.a);
	},
	function usingSchemas(t) {
		var testStr = '{id:"/dog/1",eats:{$ref:"/cat/2"}}';
		var schemas = {
			"/dog/":{prototype:{barks:true}},
			"/cat/":{prototype:{meows:true}}
		}
		var testObj = dojox.json.ref.fromJson(testStr,{
			schemas:schemas
		});
		t.t(testObj.barks);
		t.t(testObj.eats.meows);
	}
	
	/*,
	function performanceTest(t) {
		var normalishJson= '[{"id":"1",	"created":"2007-10-23T14:40:18Z","address":"somewhere","phoneNumber":"555-5555","comment":"this is great",	"firstName":"Jim",	"lastName":"Jones"},{"id":"20","created":"2008-06-03T19:45:12Z",	"firstName":"Kristopher",	"lastName":"dddddd"	},{"id":"23",	"foo":"ba=sr",	"firstName":"Jennika",	"lastName":"Zyp"	}]';
		var now = new Date().getTime();
		for(var i=0;i<1000;i++){
		}
		console.log("Just Loop",new Date().getTime()-now);
		now = new Date().getTime();
		var result;
		for(i=0;i<1000;i++){
			result = dojo.fromJson(normalishJson);
		}
		console.log("Normal fromJson",new Date().getTime()-now, result, normalishJson.length);
		now = new Date().getTime();
		for(i=0;i<1000;i++){
			result = dojox.json.ref.fromJson(normalishJson);
		}
		console.log("JSON Referencing toJson",new Date().getTime()-now, result);
	}*/
]);
