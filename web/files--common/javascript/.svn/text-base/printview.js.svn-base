/*
 * Wikidot - free wiki collaboration software
 * Copyright (c) 2008, Wikidot Inc.
 * 
 * Code licensed under the GNU Affero General Public 
 * License version 3 or later.
 *
 * For more information about licensing visit:
 * http://www.wikidot.org/license
 */

WIKIDOT.printview = {};

WIKIDOT.printview.listeners = {

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
			family = WIKIDOT.printview.ff;
		}
		//alert(family);
		body.style.fontFamily = family;
	},
	changeFontFamilyOriginal: function(e){
		var body = $("html-body");
		body.style.fontFamily = WIKIDOT.printview.ff;
		
	}
	
	
}

WIKIDOT.printview.init = function(){
	// store original font family
	OZONE.dom.onDomReady(function(){
		var body = $("html-body");
		WIKIDOT.printview.ff = 	body.style.fontFamily;
//		alert(body.style.fontFamily+' asd');
	}, "dummy-ondomready-block");
}

WIKIDOT.printview.init();