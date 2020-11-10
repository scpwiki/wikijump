

Wikijump.printview = {};

Wikijump.printview.listeners = {

	toggleSourceInfo: function(e){
		var el = $("print-head");
		if(el.style.display == "none"){
			el.style.display = "block";
		}else{
			el.style.display = "none";
		}
	},

	changeFontSize: function(e, size){
		var body = $("html-body");
		body.style.fontSize = size;
	},

	changeFontFamily: function(e, family){
		var body = $("html-body");
		if(family == 'roman'){
			family = '"Times New Roman", Times, serif';
		}
		if(family == 'original'){
			family = Wikijump.printview.ff;
		}
		//alert(family);
		body.style.fontFamily = family;
	},
	changeFontFamilyOriginal: function(e){
		var body = $("html-body");
		body.style.fontFamily = Wikijump.printview.ff;

	}


}

Wikijump.printview.init = function(){
	// store original font family
	OZONE.dom.onDomReady(function(){
		var body = $("html-body");
		Wikijump.printview.ff = 	body.style.fontFamily;
//		alert(body.style.fontFamily+' asd');
	}, "dummy-ondomready-block");
}

Wikijump.printview.init();
