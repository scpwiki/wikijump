dojo.provide("demos.fonts.src.news");
dojo.require("demos.fonts.src.pie");
dojo.require("dojo.date.locale");

function init(){
	//	get the fonts
	var euro = dojox.gfx.getVectorFont("resources/eurostyle.svg");
	var book = dojox.gfx.getVectorFont("resources/bookplate.svg");

	//	do the header and the date
	dojo.byId("date").innerHTML = dojo.date.locale.format(new Date(), { formatLength: "long", selector: "date" });

	var t = dojo.byId("title"),
		b = dojo.marginBox(t),
		txt = t.innerHTML,
		w = book.getWidth(txt, book._getSizeFactor("72px"));
	t.innerHTML = "";
	var s = dojox.gfx.createSurface(t, w, 72),
		g = s.createGroup();
	book.draw(g, 
		{ text: txt, width:w, height:60, align:"middle" },
		{ size: "72px" },
		"#181818"
	);

	if(dojo.isIE){
		t.parentNode.style.textAlign = "left";
	}

	var t = dojo.byId("subtitle"),
		txt = t.innerHTML,
		w = book.getWidth(txt, book._getSizeFactor("20px"));
	t.innerHTML = "";
	var s = dojox.gfx.createSurface(t, w, 20),
		g = s.createGroup();
	euro.draw(g,
		{ text: txt, align: "middle" },
		{ size: "20px" },
		"#333"
	);

	//	replace the rest of the headings
	dojo.forEach([ "mainHeading", "label" ], function(item){
		var t = dojo.byId(item),
			txt = t.innerHTML,
			w = euro.getWidth(txt, euro._getSizeFactor("24px"));
		t.innerHTML = "";
		var s = dojox.gfx.createSurface(t, w, 30),
			g = s.createGroup();
		euro.draw(g,
			{ text: txt },
			{ size: "24px" },
			"#333"
		);
	});

	dojo.query("#body h2").forEach(function(t){
		var txt = t.innerHTML,
			w = book.getWidth(txt, book._getSizeFactor("16px"));
		t.innerHTML = "";
		var s = dojox.gfx.createSurface(t, w, 18),
			g = s.createGroup();
		book.draw(g,
			{ text: txt },
			{ size: "16px" },
			"#000"
		);
	});

	//	copyright.
	var t = dojo.byId("bottomCopy"),
		txt = t.innerHTML,
		w = book.getWidth(txt, book._getSizeFactor("9px"));
	t.innerHTML = "";
	var s = dojox.gfx.createSurface(t, w, 10),
		g = s.createGroup();
	book.draw(g,
		{ text: txt },
		{ size: "9px" },
		"#000"
	);

	//	draw the pie chart and scale it down.
	var data = [
		{ label: "Pretty likely", value: 200 },
		{ label: "Maybe", value: 142 },
		{ label: "Not too likely", value: 76 },
		{ label: "Ain't gonna do it", value: 49 }
	];

	var p = new demos.fonts.src.pie(dojo.byId("chart"), euro, data);
	var g = p.draw();
}
dojo.addOnLoad(init);
