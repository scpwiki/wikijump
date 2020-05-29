dojo.provide("demos.fonts.src.charts");
dojo.require("demos.fonts.src.pie");

function init(){
	var font = dojox.gfx.getVectorFont("resources/eurostyle.svg");

	var set1 = [
		{ label: "Demos", value: 33382400 },
		{ label: "Dojo", value: 7602176 },
		{ label: "Dijit", value: 16642048 },
		{ label: "DojoX", value: 59453440 },
		{ label: "Util", value: 50647040 }
	];
	var set2 = [
		{ label: "Bill Keese", value: 983 },
		{ label: "Adam Peller", value: 963 },
		{ label: "Alex Russell", value: 753 },
		{ label: "Pete Higgins", value: 572 },
		{ label: "Doug Hays", value: 430 },
		{ label: "Eugene Lazutkin", value: 395 },
		{ label: "James Burke", value: 311 },
		{ label: "Neil Roberts", value: 284 }
	];
	var set3 = [
		{ label: "JavaScript", value: 170206 },
		{ label: "CSS", value: 18939 },
		{ label: "HTML", value: 41911 },
		{ label: "PHP", value: 12127 }
	];

	//	create the charts
	var pie1 = new demos.fonts.src.pie(dojo.byId("pie1"), font, set1);
	pie1.draw();
	
	var pie2 = new demos.fonts.src.pie(dojo.byId("pie2"), font, set2, {
		fill: "#aeb31d",
		main: "#636608",
		rotation: 37.5
	});
	pie2.draw();

	var pie3 = new demos.fonts.src.pie(dojo.byId("pie3"), font, set3, {
		fill: "#ffee00",
		main: "#b1e07b",
		stroke: "#4f6e2c",
		rotation: 180
	});
	pie3.draw();
}

dojo.addOnLoad(init);
