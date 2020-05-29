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

OZONE.dialog = {};

// array of objects "on screen"
OZONE.dialog.stock = new Array();

OZONE.dialog.cleanAll = function(options){
	if(!options || typeof(options.timeout) != "number"){
		timeout = 200;
	}else{ timeout = options.timeout; }
	
	setTimeout('OZONE.dialog.factory.boxcontainer().hide()', timeout);
	setTimeout('OZONE.dialog.factory.shader().hide()', timeout);
}

OZONE.dialog.factory = {
	shader: function(){
		if(OZONE.dialog.factory.stock.shader == null){
			OZONE.dialog.factory.stock.shader = new OZONE.dialog.shader();
		}
		return OZONE.dialog.factory.stock.shader;
	},
	boxcontainer: function(){
		if(OZONE.dialog.factory.stock.boxcontainer == null){
			OZONE.dialog.factory.stock.boxcontainer = new OZONE.dialog.boxcontainer2();
		}
		return OZONE.dialog.factory.stock.boxcontainer;
	}
}
OZONE.dialog.factory.stock = {};

OZONE.dialog.shader = function(){
	
	
	
	this.color = null;
	this.cssClass = null;
	
	this.setColor = function(color){
		this.color = color;
	}
	
	this.show = function(){
		var sDiv = document.getElementById("odialog-shader");
		if(sDiv != null){
			return;
		}
		sDiv = document.createElement('div');
		sDiv.id = 'odialog-shader';
		//sDiv.innerHTML = "dupa";
		var body = document.getElementsByTagName('body').item(0);
		var bodyHeigh =OZONE.visuals.bodyHeight()+50;
		var viewportHeight = YAHOO.util.Dom.getClientHeight();
		var height = Math.max(bodyHeigh,viewportHeight);
		sDiv.style.height = height+"px";
		if(this.color != null){
			sDiv.style.backgroundColor = this.color;
		}
		if(this.cssClass != null){
			sDiv.className = this.cssClass;
		} else{
			sDiv.className = "odialog-shader";
		}
		
		// extra iframe for stupid browsers
		
		if(window.navigator.userAgent.match(/msie/i)){
			var sIfr = document.createElement('iframe');
			sIfr.id="odialog-shader-iframe";
			sIfr.src="/files--common/misc/blank.html";
	//		sIfr.scrolling=0;
			sIfr.frameBorder=0;
			sIfr.className='odialog-shader-iframe';
			sIfr.style.height = height+"px";
		
			body.appendChild(sIfr);
	//		alert('iframe');
		}
		body.appendChild(sDiv);
	
	
		
	}
	this.hide = function(){
		var body = document.getElementsByTagName('body').item(0);
		var sDiv = $("odialog-shader");
		var sIfr = $("odialog-shader-iframe");
		if(sDiv != null){
			body.removeChild(sDiv);
		}
		if(sIfr != null){
			body.removeChild(sIfr);
		}
	}		

}

OZONE.dialog.boxcontainer2 = function(){
	
	this.mDiv = null;
	this.cDiv = null;
	
	
	this.init = function(){
		//alert("init");
		var el = $("odialog-container");
		if(!el){
			el = document.createElement('div');
			el.id = "odialog-container";
			var body = document.getElementsByTagName('body').item(0);
			body.appendChild(el);
			//el.style.visibility="hidden";
			this.mDiv = el;
		}
		el.style.display = "block";
	}
	
	this.setContent = function (content){
		this.clearContent();
		//alert(typeof content);
		if(typeof content == 'string'){
			this.mDiv.innerHTML = content;
		}else{
			//object
			this.mDiv.appendChild(content);
		}
		OZONE.utils.formatDates(this.mDiv);
		OZONE.dialog.hovertip.dominit(this.mDiv,{delay: 300});
		var cDiv = this.mDiv.getElementsByTagName('div').item(0);
		cDiv.style.visibility = "hidden";
		this.cDiv = cDiv;	
		this.mDiv.style.display="block";
		// center by default
		this.centerContent();
		
	//	this.effect = new fx.Height(cDiv, {duration: 300});
	//	cDiv.style.height = "0px";
//		this.effect = new fx.Opacity(cDiv, {duration: 300});
//		this.effect.setOpacity(0);

		//add drag&dop
		//if(cDiv.id == null){
			cDiv.id = "owindow-1";
		//}
		//alert(this.cDiv.id);
		
		//var tDiv = cDiv.getElementsByTagName('div').item(0);
		//if(tDiv && tDiv.className == "title"){
		//	tDiv.id="handle-1";
		//	dd1.setHandleElId(tDiv.id);
		//}
		var tDivs = cDiv.getElementsByTagName('div');
		var i;
		for(i in tDivs){
			if(tDivs[i].className == "title"){
				tDivs[i].id="ohandle-1";
				var dd1 = new YAHOO.util.DD(this.cDiv.id);
				dd1.setHandleElId(tDivs[i].id);
			}
			if(tDivs[i].className == "close"){
				//tDivs[i].id="oclose-1";
				YAHOO.util.Event.addListener(tDivs[i], "click", OZONE.dialog.cleanAll);
			}
		}
	}
	this.attachDD = function(){
		var tDivs = this.cDiv.getElementsByTagName('div');
		var i;
		for(i in tDivs){
			if(tDivs[i].className == "title"){
				tDivs[i].id="ohandle-1";
				var dd1 = new YAHOO.util.DD(this.cDiv.id);
				dd1.setHandleElId(tDivs[i].id);
			}
			if(tDivs[i].className == "close"){
				//tDivs[i].id="oclose-1";
				YAHOO.util.Event.addListener(tDivs[i], "click", OZONE.dialog.cleanAll);
			}
		}
	}
	
	this.clearContent = function(){
		this.cDiv = null;
		this.mDiv.innerHTML = "";
	}
	
	this.centerContent = function(){
		//alert("centering");
		var cDiv = this.cDiv;
		var height = cDiv.offsetHeight;
		var width = cDiv.offsetWidth;
		var vpHeight = YAHOO.util.Dom.getClientHeight();
		var vpWidth = YAHOO.util.Dom.getClientWidth();
		
		var posX = Math.max((vpWidth - width)*0.5,0);
		var posY = Math.max(OZONE.visuals.scrollOffsetY() + (vpHeight - height)*0.5,0);
		
		YAHOO.util.Dom.setXY(cDiv, [posX, posY]);
		
		
	}
	
	this.setContentObject = function(object){
		this.mDiv.appendChild(object);
	}
	
	this.showContent = function (options){
		this.mDiv.style.display="block";
		
		if(options && options.smooth == true){
			var ef = new fx.Opacity(this.cDiv, {duration: 300});
			ef.setOpacity(0);
			this.cDiv.style.visibility = "visible";
			ef.custom(0,1);
		}else{
			this.cDiv.style.visibility = "visible";
		}
		
		
	},
	
	this.hideContent = function(){
		//this.effect.custom(1,0);
		this.cDiv.style.visibility = "hidden";
	}
	
	this.hide = function(options){
		if(options && options.smooth == true){
			var ef = new fx.Opacity(this.cDiv, {duration: 300});
			ef.setOpacity(1);
			//this.cDiv.style.visibility = "visible";
			ef.custom(1,0);
		}
		this.clearContent();
		// ???
		$("odialog-container").style.display = "none";
	}
	
	this.clickOutsideToHide = function(val){
		// atach a listener to shader...
		YAHOO.util.Event.addListener("odialog-shader", "click",OZONE.dialog.cleanAll);
	}
	
	this.changeContent = function (content){
		this.setContent(content);
		this.showContent();
	}
	
	this.init();
}

OZONE.dialog.hovertip = {
	
	container: null,
	bindings: new Array(),
	
	
	init: function(){
		var el = $('odialog-hovertips');
		if(!el){
//			alert("init");
			el = document.createElement('div');
			el.id='odialog-hovertips';
		//	el.style.display = "none";
			el.style.position="absolute";
			el.style.zIndex= 100
			el.style.top = 0;
			//el.style.visibility="hidden";
			el.style.width="100%";
			var body = document.getElementsByTagName('body').item(0);
			body.appendChild(el);
			OZONE.dialog.hovertip.container = el;
			//el.innerHTML = "hovertips - test";
		}
	},
	
	
	makeTip: function(element, options){
		
		if( typeof element != "string" && element.length > 0){
//			alert(element.id+' '+element.length+ typeof element);
			// iterate over elements
			for(var i=0; i<element.length; i++){
				OZONE.dialog.hovertip.makeTip(element[i],options);
			}
		}
		
		// options can be: text (text) or context element id (context)
		OZONE.dialog.hovertip.init(); // just for sure
		var body = document.getElementsByTagName('body').item(0); 
		var el = $(element);
		if(!el){return;}
		if(el.hovertip){return;}
		var tipEl;
			
		if(options && options.context){
			tipEl = $(options.context);
			// move to 'odialog-hovertips'
			if(!tipEl){ return;} // return if not valid element
			
		}else if(options && options.text){
			// create a new div
			tipEl = document.createElement('div');
			tipEl.innerHTML = '<div class="content">'+options.text+'</div>';
			
		}else{
			// init from the "title" attribute
			var title;
			if(el.attributes){
				for( var x = 0; x < el.attributes.length; x++ ) {
					if( el.attributes[x].nodeName.toLowerCase() == 'title' ) {
						title = el.attributes[x].nodeValue;
						el.attributes[x].nodeValue = '';
					}
				}
			}
		
			if(!title){
				return;
			}
			tipEl = document.createElement('div');
			tipEl.innerHTML = '<div class="content">'+title+'</div>';
		
		}
		
		// check for the "hovertip" class
		if(!tipEl.className.match(/hovertip/)){
			tipEl.className = 'hovertip '+tipEl.className;
		}
		if(options){
			el.hovertipOptions = options;
		}
		if(options && options.style){
			for(var key in options.style){
				tipEl.style[key] = options.style[key];
			}
		}
		
		// fix if not "content" div inside.
		var subDivs = tipEl.getElementsByTagName('div');
		var hasContent = false;
		for(var i=0; i<subDivs.length;i++){
			if(YAHOO.util.Dom.hasClass(subDivs[i], 'content')){
				hasContent = true;
			}
		}
		if(!hasContent){tipEl.innerHTML = '<div class="content">'+tipEl.innerHTML+'</div>';}
		
		
		// make sure some properties are set properly
		el.hovertip = tipEl;
		var eff = new fx.Opacity(el.hovertip, {duration:300})	;
		el.hovertipEffect = eff;
		
		tipEl.style.position = "absolute";
		tipEl.style.display = "none";

		// for debugging
		tipEl.style.border = "1px solid black";
		
		if(el.tagName.toLowerCase() != 'a' && (!options || !options.noCursorHelp)){
			el.style.cursor = "help";
		}
		// moving along...
		$('odialog-hovertips').appendChild(tipEl);
		
		// somehow make a binding now
		OZONE.dialog.hovertip.bindings.push([el, tipEl]);

		YAHOO.util.Event.addListener(el, "mousemove",OZONE.dialog.hovertip._mousemove);
		YAHOO.util.Event.addListener(el, "mouseout",OZONE.dialog.hovertip._mouseout);
		YAHOO.util.Event.addListener(el, "mouseover",OZONE.dialog.hovertip._mouseover);
		return;
	},
	
	_mouseover: function(e){
		var el = YAHOO.util.Event.getTarget(e);
		
		var tipEl = el.hovertip;//tipBin[1];
		tipEl.style.visibility = "hidden";
		tipEl.style.opacity=0;
		tipEl.style.display="block";
		//var eff = new fx.Opacity(el.hovertip, {duration:300})	;
		var options = el.hovertipOptions;
		var eff = el.hovertipEffect;
		// position to (0,0) to avoid glitches
		YAHOO.util.Dom.setXY(el.hovertip, [0, 0]);	
		//eff.setOpacity(0);
		//return;
		// trigger mousemove too!
		OZONE.dialog.hovertip._mousemove(e);
		
		if(options && options.delay){
			OZONE.dialog.tmpeff = eff;
			
			setTimeout('if(OZONE.dialog.tmpeff.el.style.opacity==0)OZONE.dialog.tmpeff.custom(0,1)', options.delay);
		} else {
			
			eff.custom(0,1);
		}
		
	},
	
	_mousemove: function(e){
		
		// get the element
		var el = YAHOO.util.Event.getTarget(e);
	
		var tipEl = el.hovertip;//tipBin[1];
		//tipEl.style.visibility = "hidden";
		// position and display the tip
		
		// get mouse position
		var posx = 0;
		var posy = 0;
		if (!e) var e = window.event;
		if (e.pageX || e.pageY)
		{
			posx = e.pageX;
			posy = e.pageY;
		}
		else if (e.clientX || e.clientY)
		{
			posx = e.clientX + document.documentElement.scrollLeft;
			posy = e.clientY + document.documentElement.scrollTop;
		}
//		alert(posx+" "+posy);
		// position the tipEl
			
		// now calculate where to position the tip box
//		alert(tipEl.offsetWidth);		

		// get viewport size
		var vHeight = YAHOO.util.Dom.getViewportHeight();
		var vWidth = YAHOO.util.Dom.getViewportWidth();
		var tipElHeight = tipEl.offsetHeight;
		var tipElWidth = tipEl.offsetWidth;
		
		var border = 20; // border (whitearea) size
		
		if(el.hovertipOptions && el.hovertipOptions.smartWidthLimit){
//			alert('smart');
			var vlimit = el.hovertipOptions.smartWidthLimit;
			if(tipElWidth> vlimit * vWidth){
				tipEl.style.width = vlimit * vWidth+'px';
			}
		}
		
		// not to go outsite right/bottom border
		// assume sizes are considerably smaller than the 
		// viewport size!
		if(el.hovertipOptions && el.hovertipOptions.valign){
			switch(el.hovertipOptions.valign){
				case 'center': 
					if(vHeight - e.clientY < tipElHeight + 2*border && e.clientY > tipElHeight + 1.5*border){
						posy -= tipElHeight + 1.5*border;
					}
					posy +=border;
					posx = e.clientX - tipElWidth*0.5;
					if(posx+tipElWidth > vWidth-border){posx = vWidth-tipElWidth - border;}
					if(posx<border){posx = border;}
			}
		} else{
			if(vWidth - e.clientX < tipElWidth + 2*border && e.clientX > tipElWidth + 1.5*border){
				posx -= tipElWidth + 1.5*border;
			}
			if(vHeight - e.clientY < tipElHeight + 2*border && e.clientY > tipElHeight + 1.5*border){
				posy -= tipElHeight + 1.5*border;
			}
			posx +=border;
			posy +=border;
		}
		YAHOO.util.Dom.setXY(tipEl, [posx, posy]);	
	
	
		
	},
	
	_mouseout: function(e){
		// just hide it!
		var el = YAHOO.util.Event.getTarget(e);
		
		var tipEl = el.hovertip;
		tipEl.style.display = "none";
		
	},
	
	dominit: function(topEl, options){
		// parse the DOM tree and find pairs:
		// someid, someid-hovertip
		// and make the tip binding
		OZONE.dialog.hovertip.init(); // just for sure
		var allDivs;
		if(topEl){
			allDivs = $(topEl).getElementsByTagName('div');
		} else{
			allDivs = document.getElementsByTagName('div');
		}
		var tipEls = new Array();
		for(var i=0; i<allDivs.length; i++){
			if(allDivs[i].id.match(/\-hovertip$/)){
				tipEls.push(allDivs[i]);
			}
		}
		for(var i=0; i<tipEls.length; i++){
			var tipEl = tipEls[i];
			var elId = tipEl.id.replace(/\-hovertip$/, '');
			var el = $(elId);
			if(el){
			//	alert("found "+elId);
				if(!options){var options = new Object();}
				options.context = tipEl;
				OZONE.dialog.hovertip.makeTip(el, options);
			}
		}
	},
	
	hideAll: function(){
		var cont = $('odialog-hovertips');
		if(cont){
			var els = cont.getElementsByTagName('div');
			for(var i=0; i<els.length; i++){
				if(els[i].className.match(/hovertip/)){
					els[i].style.display="none";
				}
			}
		}
	}
	
	
}

/**
 *  ready to use components
 */
OZONE.dialogs = {}
	
OZONE.dialogs.Base = function(){}

OZONE.dialogs.Base.prototype = {
	
	
	initialize: function(){
		this.templateBase = '/common--dialogs/';
		this.template = '';
		this.title = null;
		this.buttons = new Array();
		this.buttonObjects = new Array();
		this.clickOutsideToClose = false;
		this.smooth = false;
		this.focusButton = null;
		this.buttonListeners = new Object();
		this.windowClass = '';
		this.content = '';
		this.windowDiv = null;
		this.fixODate = true;
		this.style = new Object();
	},
	
	/**
	 * e.g. ['cancel','OK', 'next', 'no', 'yes'] etc.
	 */
	setButtons: function(arrasy){
		
	},
	addButtonListener: function(buttonLabel, eventListener, oScope){
		this.buttonListeners[buttonLabel] = eventListener;
	},
	
	show: function(){
		var windowDiv = document.createElement('div');
		this.windowDiv = windowDiv;
		windowDiv.className="owindow "+this.windowClass;
		var styleProperty;
		for(styleProperty in this.style){
			windowDiv.style[styleProperty] = this.style[styleProperty];
		//	alert(styleProperty);
		}
		
		// in there is a div class="content" - just place it inside, do not render
		var tmpdiv = document.createElement('div');
		tmpdiv.innerHTML = this.content;
		if(tmpdiv.getElementsByTagName('div').item(0) && YAHOO.util.Dom.hasClass(tmpdiv.getElementsByTagName('div').item(0), "owindow")){
			windowDiv = tmpdiv.getElementsByTagName('div').item(0);
//			alert("??");
		}else if(YAHOO.util.Dom.getElementsByClassName("content", "div", tmpdiv).length==1){
			windowDiv.innerHTML = tmpdiv.innerHTML;
		}else{
		
			if(this.title != null){
				var titleDiv = document.createElement('div');
				titleDiv.className = 'title';
				titleDiv.innerHTML = this.title;
				windowDiv.appendChild(titleDiv);
			}
			var contentDiv = document.createElement('div');
			contentDiv.className = 'content';
			contentDiv.innerHTML = this.content;
			if(this.fixODate){
				OZONE.utils.formatDates(contentDiv);
			}
			windowDiv.appendChild(contentDiv);
			
			if(this.buttons.length>0){
				var buttonBar =  document.createElement('div');
				buttonBar.className ='button-bar';
				for(var j=0; j<this.buttons.length; j++){	
					var blabel = this.buttons[j];
					var button = document.createElement('a');
					//button.type="button";
					//button.value=blabel;
					button.href="javascript:;";
					button.innerHTML = ogettext(blabel);
					button.className='button button-'+blabel.toLowerCase().replace(/ /g, '-'); // if you use standard names it is a gain here.
					if(this.buttonListeners[blabel]){
						YAHOO.util.Event.addListener(button, 'click', this.buttonListeners[blabel], this, true);
					}
					buttonBar.appendChild(button);
					this.buttonObjects[blabel] = button;
				}
				windowDiv.appendChild(buttonBar);
			}
		}
		OZONE.dialog.factory.shader().show();
		var container = OZONE.dialog.factory.boxcontainer();
		container.setContent(windowDiv);
		if(this.smooth == true){
			container.showContent({smooth: true});
		}else{
			container.showContent();
		}
		if(this.clickOutsideToClose){
			container.clickOutsideToHide();
		}
		if(this.focusButton && this.buttonObjects[this.focusButton]){
//			this.oldFocus = window.????????
			this.buttonObjects[this.focusButton].focus();
			// hack to make Opera not select the innerhtml
			// ??????????
			
		}
		
	},

	hide: function(){
		if(this.smooth == true){
			var ef = new fx.Opacity(this.windowDiv, {duration: 100});
			ef.custom(1,0);
		}
	},


	// cleans shader too
	close: function(){
		this.hide();
		OZONE.dialog.cleanAll({timeout: 200});
	}
	
}

OZONE.dialogs.SmallInfoBox  = Class.create();
OZONE.dialogs.SmallInfoBox.prototype = Object.extend(new OZONE.dialogs.Base(), {
	initialize: function(){
		OZONE.dialogs.Base.prototype.initialize.call(this);
		this.smooth = true;
	 	this.windowClass = "o-infobox";
	}
});

OZONE.dialogs.WaitBox  = Class.create();
OZONE.dialogs.WaitBox.prototype = Object.extend(new OZONE.dialogs.Base(), {
	initialize: function(){
		OZONE.dialogs.Base.prototype.initialize.call(this);
		this.smooth = true;
	 	this.windowClass = "owait";
	}
});

OZONE.dialogs.SuccessBox  = Class.create();
OZONE.dialogs.SuccessBox.prototype = Object.extend(new OZONE.dialogs.Base(), {
	initialize: function(){
		OZONE.dialogs.Base.prototype.initialize.call(this);
		this.smooth = true;
	 	this.windowClass = "osuccess";
	 	this.timeout=1500;
	}
});
OZONE.dialogs.SuccessBox.prototype.show = function(){
	OZONE.dialogs.Base.prototype.show.call(this);
	if(this.timeout){
		setTimeout('OZONE.dialog.cleanAll()', this.timeout);
	}
}

OZONE.dialogs.ErrorDialog  = Class.create();
OZONE.dialogs.ErrorDialog.prototype = Object.extend(new OZONE.dialogs.Base(), {
	initialize: function(){
		OZONE.dialogs.Base.prototype.initialize.call(this);
		//this.smooth = true; // content should be simple so make it fade...
	 	this.windowClass = "error";
	 	this.title = "Error";
	 	var lab = 'close message';
	 	this.buttons = [lab];
		this.addButtonListener(lab, this.close);
		this.focusButton = lab;
	}
});

OZONE.dialogs.ConfirmationDialog  = Class.create();
OZONE.dialogs.ConfirmationDialog.prototype = Object.extend(new OZONE.dialogs.Base(), {
	initialize: function(){
		OZONE.dialogs.Base.prototype.initialize.call(this);
		//this.smooth = true; // content should be simple so make it fade...
	 	this.windowClass = "confirmation";
	 	this.title = "Confirmation";
	}
});

OZONE.dialogs.SuccessDialog  = Class.create();
OZONE.dialogs.SuccessDialog.prototype = Object.extend(new OZONE.dialogs.Base(), {
	initialize: function(){
		OZONE.dialogs.Base.prototype.initialize.call(this);
		this.smooth = true; // content should be simple so make it fade...
	 	this.windowClass = "confirm";
	 	this.title = "Success";
	 	this.buttons = ['close message'];
		this.addButtonListener('close message', this.close);
		this.focusButton = 'close message';
	}
});

OZONE.dialogs.InfoDialog  = Class.create();
OZONE.dialogs.InfoDialog.prototype = Object.extend(new OZONE.dialogs.Base(), {
	initialize: function(){
		OZONE.dialogs.Base.prototype.initialize.call(this);
		this.smooth = true; // content should be simple so make it fade...
	 	this.windowClass = "info";
	 	this.title = " ";
	 	this.buttons = ['close window'];
		this.addButtonListener('close window', this.close);
		this.focusButton = 'close window';
	}
});


/**
 * Just a basic dialog... use this if unsure.
 */
OZONE.dialogs.Dialog = Class.create();
OZONE.dialogs.Dialog.prototype = Object.extend(new OZONE.dialogs.Base(), {
	initialize: function(){
		//alert("iint");
		OZONE.dialogs.Base.prototype.initialize.call(this);
		this.title='';
	}
});



function exinfo2(){
	
	this.show = function(){
		var shader = OZONE.dialog.factory.shader();
		shader.show();
		var container = OZONE.dialog.factory.boxcontainer();
		
		
		//container.show();
		container.setContent('<div class="box444">DUPA</div>');
		container.showContent();
	}
}

// test begins... here!
function listener1(){
	var shader = new OZONE.dialog.shader();
//	shader.color="black";
	shader.show();
	
}

function listener2(){
	e = new exinfo2();
	e.show();
}

function testdialog(){
	//alert('te');	
	var bd = new OZONE.dialogs.Base();
	bd.template = 'Warning';
//	bd.title="to jest warning!!!";
	bd.content = "dupowy content";
	bd.buttons = ['cancel', 'Ok'];
	bd.addButtonListener('cancel', bd.close);
	bd.smooth = true;
	bd.show();
}

function testdialog2(){
//	var t2 = new OZONE.dialogs.SmallInfoBox();
	var t2 = new OZONE.dialogs.ErrorDialog();
	t2.content="<h1>Error processing template...</h1>test"	;
	t2.show();

}

function testdialog3(){
//	var t2 = new OZONE.dialogs.SmallInfoBox();
	var t2 = new OZONE.dialogs.SuccessBox();
	t2.content="Loading file..."	;
	t2.timeout=1000;
	t2.show();

}


//var dd3 = new YAHOO.util.DD("header");