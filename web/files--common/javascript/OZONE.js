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

var OZONE = function(){}

OZONE.ajax = {

	_callbackArray: new Array(),
	_callbackArrayIndex: 0,
	_javascriptLoadLock: false,
	
	/**
	 * arg - extra parameter passed to the callback as a second parameter
	 */
	requestModule: function(moduleName, parameters, callback, arg, options){
		OZONE.visuals.cursorWait();
		if(parameters==null){
			parameters = new Object();
		}
		if(moduleName == null || moduleName == ""){
			moduleName = "Empty";
		}
		parameters["moduleName"] = moduleName;
	
		if(options && options.clearRequestQueue){
			OZONE.ajax._callbackArray = new Array();
		}
	
		var callbackIndex = OZONE.ajax._callbackArrayIndex++;
		OZONE.ajax._callbackArray[callbackIndex] = {callback: callback, arg: arg};
		
		parameters['callbackIndex'] = callbackIndex;
		
		// add token information
		var token = OZONE.utils.getCookie("wikidot_token7");
		if(token == null){
			alert("Error processing the request.\n\n" +
					"You have no valid security token which is required to prevent " +
					"identity theft.\n" +
					"Please enable cookies " +
					"in your browser if you have this option disabled " +
					"and reload the page.");
			OZONE.visuals.cursorClear();
			return;		
		}
		parameters['wikidot_token7'] = token;
		
		var postdata = OZONE.utils.arrayToPostData(parameters);
		var internalCallback = OZONE.ajax.requestModuleCallback;
		YAHOO.util.Connect.asyncRequest('POST','/ajax-module-connector.php',internalCallback,postdata);
		
	},

	parseResponse: function(jsonResponse){
		res = JSON.parse(jsonResponse);
        if(!res){
        	alert(jsonResponse.replace(/\r?\n/g,' '));
        }
		
		return res;
	},
	
	requestModuleCallback: {
		success: function(rObj){
			// process response
			var response = OZONE.ajax.parseResponse(rObj.responseText);
			
			if(response.status=='wrong_token7'){
				// TODO: De-Wikidot.com-ize - change
				alert('wikidot.com security error:\n\n' +
						'Your authentication token in the request is not valid. ' +
						'Please enable cookies in your browser and try to repeat the action.\n\n' +
						'If you see this message on the page not associated with the wikidot.com ' +
						'wiki hosting it probably means an indentity theft attempt or ' +
						'improper use of wikidot.com service.');
				OZONE.visuals.cursorClear();
				return;
			}
			
			var callbackIndex = response.callbackIndex;
			if(callbackIndex == null){
				OZONE.visuals.cursorClear();
				OZONE.dialog.cleanAll();	
			}
			if(!OZONE.ajax._callbackArray[callbackIndex]){
				return;
			}
			var callback = OZONE.ajax._callbackArray[callbackIndex]['callback'];
			if(!callback){
				alert("internal: callback error");
			}
			var arg = OZONE.ajax._callbackArray[callbackIndex]['arg'];
			// call callback
			
			if(arg != null){
				callback(response, arg);
			}else{
				callback(response);
			}
			
			// attach javascript (if any)
			if(response.jsInclude != null){
				for(var index=0; index<response.jsInclude.length; index++) {
					OZONE.utils.addJavascriptUrl(response.jsInclude[index]);
				}
			}
			if(response.cssInclude != null){
				
				for(var index=0; index<response.cssInclude.length; index++) {
					OZONE.utils.addStyleUrl(response.cssInclude[index]);
				}
			}
			OZONE.visuals.cursorClear();
		},
		failure: function(rObj){
			alert("The ajax request failed. Please check your internet connection or\n" +
					"report a bug if the error repeats during your work."+"\ncode:"+rObj.status);

			OZONE.visuals.cursorClear();
			OZONE.dialog.cleanAll();
		}	
	
	},
	
	requestQuickModule: function(moduleName, parameters, callback){
		if(parameters==null){
			parameters = new Object();
		}
		if(moduleName == null || moduleName == ""){
			alert('Quick module name empty.');
		}
	
		var callbackIndex = OZONE.ajax._callbackArrayIndex++;
		OZONE.ajax._callbackArray[callbackIndex] = callback;
		
		parameters['callbackIndex'] = callbackIndex;
		
		var postdata = JSON.stringify(parameters);
		var internalCallback = OZONE.ajax.requestQuickModuleCallback;
		YAHOO.util.Connect.asyncRequest('POST','/quickmodule.php?module='+moduleName,internalCallback,postdata);
		
	},

	parseResponse: function(jsonResponse){
		res = JSON.parse(jsonResponse);
        if(!res){
        		alert(jsonResponse.replace(/\r?\n/g,' '));
        }
		
		return res;
	},
	
	requestQuickModuleCallback: {
		success: function(rObj){
			// process response
			var response = OZONE.ajax.parseResponse(rObj.responseText);
			var callbackIndex = response.callbackIndex;
			var callback = OZONE.ajax._callbackArray[callbackIndex];
			callback(response);
		},
		failure: function(rObj){
			alert("The ajax request failed. Please check your internet connection or\n" +
					"report a bug if the error repeats during your work.");
		}	
	
	}
	
	
}
  

OZONE.utils = {
	formToArray: function(form){
		form = $(form);
		if(form == null) return;

		// process different form elements (traverse)
		var parms = new Object();
		for(i=0;i<form.length;i++){
			var element = form.elements[i];
			var type = element.type;
			if(type == "text" || type=="hidden" || type=="password" || type=="select-one"||type=="textarea"){
				parms[element.name]=element.value;
			}
			if(type=="checkbox" && element.checked==true){
				parms[element.name]="on";
			}
			if(type=="radio" && element.checked==true){
				parms[element.name]=element.value;
			}

		}
		return parms;

	},

	arrayToPostData: function(ar){
		if(ar == null) {return null;}
		
		var varsString = "";
		var value;
		for(key in ar){
			value  = encodeURIComponent(ar[key]);
		 	varsString += '&' + key + '=' + value;
		 	
		}
		if (varsString.length > 0) {
        		varsString = varsString.substring(1); // chomp initial '&'
      	}
      	
      	//try with json... TODO!
		 return varsString;
	},

	addJavascriptUrl: function(url, onLoadListener, noReload){
		if(OZONE.utils._javascripLoadLock 
			&& (new Date()).getTime() < OZONE.utils._javascripLoadLock + 2000){	
			setTimeout(function(){
				OZONE.utils.addJavascriptUrl(url, onLoadListener, noReload);
			}, 50);
			return;
		}
		
		OZONE.utils._javascripLoadLock = false;
		
		var head = document.getElementsByTagName("head").item(0);
	 	var scripts=head.getElementsByTagName("script");
	 	for(i=0;i<scripts.length;i++){
	 		if(scripts[i].getAttribute("src")==url) {
	 			// return
				if(noReload){
					if(onLoadListener){
						onLoadListener();
					}	
					return;
				}
	 			head.removeChild(scripts[i]);
	 		};
	 	}

		// check if not already included
		OZONE.utils._javascripLoadLock = (new Date()).getTime();
		var mys = document.createElement('script');
		mys.setAttribute('type','text/javascript');
		mys.setAttribute('src',url);
		if(YAHOO.env.ua.ie){
		   	mys.onreadystatechange= function () {
     			if (this.readyState == 'complete' || this.readyState == 'loaded') {
					mys.onreadystatechange = null;
					OZONE.utils._javascripLoadLock = false;
					if(onLoadListener){
						onLoadListener.call();
					}
   				}
			}
		}else{
			YAHOO.util.Event.addListener(mys, "load", function(){
				OZONE.utils._javascripLoadLock = false;
				if(onLoadListener){
					onLoadListener.call();
				}
			});
	   	}
		head.appendChild(mys);
	},
	
	addStyleUrl: function(url, onLoadListener, noReload){
		var head = document.getElementsByTagName("head").item(0);
		var styles=head.getElementsByTagName("link");
		for(i=0;i<styles.length;i++){
	 		if(styles[i].type="text/css" && styles[i].getAttribute("src")==url) {
				if(noReload){
					if(onLoadListener){
						onLoadListener();
					}	
					return;
				}
	 			head.removeChild(styles[i]);
	 		};
	 	}
		
		var mys = document.createElement('link');
		mys.rel="stylesheet";
		mys.type = "text/css";
		mys.href= url;
		if(onLoadListener){
	   		YAHOO.util.Event.addListener(mys, "load", onLoadListener);
	   	}
		head.appendChild(mys);
	},
	
	setInnerHTMLContent: function(elementId, content){
		var el = $(elementId);
		if(el){
			el.innerHTML = content;	
			OZONE.utils.formatDates(el);
			OZONE.dialog.hovertip.dominit(el);	
		}
	},
	
	disableEnterKey: function(e){
		
		     var key;
		
			// disable for textareas!
			var tg = (e.target) ? e.target : e.srcElement;
			if(tg.tagName == "TEXTAREA"){
				return true;
			}
			
		     if(window.event)
		          key = window.event.keyCode;     //IE
		     else
		          key = e.which;     //firefox
		
		     if(key == 13)
		          return false;
		     else
		          return true;
		},
	escapeHtml: function(text){
		if(text == null || text == ''){
			return '';
		}
		return text.split("&").join("&amp;").split("<").join("&lt;").split(">").join("&gt;");
	},
	unescapeHtml: function(text){
		if(text == null || text == ''){
			return '';
		}
		return text.split("&gt;").join(">").split("&lt;").join("<").split("&amp;").join("&");
	},
	
	formatDates: function(topElement){
		var monthNames =new Array('January','February','March','April','May','June','July','August','September','October','November','December');
		var monthNamesShort =new Array('Jan','Feb','Mar','Apr','May','Jun','Jul','Aug','Sep','Oct','Nov','Dec');
		var dayNames = new Array('Sunday','Monday','Tuesday','Wednesday','Thursday','Friday','Saturday');
		var dayNamesShort = new Array('Sun','Mon','Tue','Wed','Thu','Fri','Sat');
		topElement = $(topElement);
		var dstring="";
		var dates = YAHOO.util.Dom.getElementsByClassName('odate', 'span', topElement);	
		for(i = 0; i<dates.length; i++){
			var	inner = dates[i].innerHTML;
			if(inner.match(/^[0-9]+$/)){
				var timestamp = inner;
				var date = new Date();
				date.setTime(timestamp*1000);
				dstring = date.toLocaleString();
			}
			if(inner.match(/^[0-9]+\s*\|.*$/)){
				var nu;
				// with formatting
				var timestamp = inner.replace(/^([0-9]+)\s*\|.*/, "$1");
				var format = inner.replace(/^[0-9]+\s*\|\s*(.*?)(?:\|(.*))?$/, "$1");
				var options = inner.replace(/^[0-9]+\s*\|\s*(.*?)(?:\|(.*))?$/, "$2");
				dates[i].timestamp = timestamp;
				
			//	alert(options)
				var date = new Date();
				date.setTime(timestamp*1000);
				dstring = format;
				
				dstring = dstring.replace(/%r/g, "%I:%M:%S %p");
				dstring = dstring.replace(/%R/g, "%H:%M");
				dstring = dstring.replace(/%T/g, "%H:%M:%S");
				dstring = dstring.replace(/%D/g, "%m/%d/%y");
				
				// %a for abbreviated weekday name
				dstring = dstring.replace(/%a/g, dayNamesShort[date.getDay()]);
				// %A for full weekday name
				dstring = dstring.replace(/%A/g, dayNames[date.getDay()]);
				// %b for short month name
				dstring = dstring.replace(/%b/g, monthNamesShort[date.getMonth()]);
				// %B for full month name
				dstring = dstring.replace(/%B/g, monthNames[date.getMonth()]);
				// %c for local date representation
				dstring = dstring.replace(/%c/g, date.toLocaleString());
				// %d for the day of the month 02
				nu = date.getDate();
				dstring = dstring.replace(/%d/g, (nu < 10) ? '0' + nu : nu);
				// %e for the day of the month 2
				dstring = dstring.replace(/%e/g, date.getDate());
				var hours = date.getHours();
				// %H for hours 00-23
				dstring = dstring.replace(/%H/g, (hours < 10) ? '0' + hours : hours);
				// %I for hours 00-12
				var hours12 = (hours-1)%12 +1 ;
				dstring = dstring.replace(/%I/g, (hours12 < 10) ? '0' + hours12 : hours12);
				var month = date.getMonth();
				dstring = dstring.replace(/%m/g, (month+1 < 10) ? '0' + (month+1) : month+1);
				var minute = date.getMinutes();
				// %M for minutes
				dstring = dstring.replace(/%M/g, (minute < 10) ? '0' + minute : minute);
				dstring = dstring.replace(/%p/g, (hours < 12) ? 'AM' : 'PM');
				
				var seconds = date.getSeconds();
				dstring = dstring.replace(/%S/g, (seconds < 10) ? '0' + seconds : seconds);
				var y = date.getYear();
				y = (y < 10) ? '0' + y : ''+y;
				dstring = dstring.replace(/%y/g, y.substring(y.length-2));
				var yc = date.getFullYear();
				dstring = dstring.replace(/%Y/g, yc);
				
				if(dstring.match(/%z/i)){
					var zone;
					// try to get zone from locale date string
					zone = date.toLocaleString().replace(/^.*?([A-Z]{3,}(?:\+[0-9]+)?).*$/, "$1")
					if(zone == date.toLocaleString()){
						var zoneoffset = date.getTimezoneOffset();
						zoneoffset = -zoneoffset/60;
						zoneoffset = ((zoneoffset < 10) ? '0' + zoneoffset : zoneoffset) + '00';
						zone = (zoneoffset>0)? '+'+zoneoffset: '-'+zoneoffset;
						
					}
					dstring = dstring.replace(/%z/ig, zone);
				}
				if(dstring.match(/%O/)||options.match(/agohover/)){
					// time ago
					var secAgo = OZONE.request.timestamp - timestamp;
					secAgo += Math.floor(((new Date()).getTime() - OZONE.request.date.getTime())*0.001);
					var agoString = OZONE.utils.calculateDateAgo(secAgo);
					
					dstring = dstring.replace(/%O/, agoString);
					if(options.match(/agohover/)){
						var hovertext = agoString+' ago';
						OZONE.dialog.hovertip.makeTip(dates[i], {text: hovertext, style: {width: 'auto'}});
						YAHOO.util.Event.addListener(dates[i], "mouseover", function(e){
							var secAgo = OZONE.request.timestamp - this.timestamp;
							secAgo += Math.floor(((new Date()).getTime() - OZONE.request.date.getTime())*0.001);
							var agoString = OZONE.utils.calculateDateAgo(secAgo);
							this.hovertip.getElementsByTagName('div').item(0).innerHTML = agoString+' '+ogettext('ago');
						});
					}
				}
			}
			if(dstring){
				dates[i].innerHTML = dstring;
				dates[i].style.visibility = "visible";
				dates[i].style.display = "inline";
			}
		}
	},
	
	calculateDateAgo: function(secAgo){
		var agoString;
		if(secAgo >= 60*60*24){
			var days = Math.floor(secAgo/(60*60*24));
			agoString = ''+days+' '+((days)>1?ogettext('days'):ogettext('day')); 
		} else if(secAgo >= 60*60){
			var hours = Math.floor(secAgo/(60*60));
			agoString = ''+hours+' '+((hours)>1?ogettext('hours'):ogettext('hour')); 
		} else if(secAgo >= 60){
			var minutes = Math.floor(secAgo/60);
			agoString = ''+minutes+' '+((minutes)>1?ogettext('minutes'):ogettext('minute')); 
		}else{
			if(secAgo == 0) {secAgo++;}
			agoString = ''+secAgo+' '+((secAgo)>1?ogettext('seconds'):ogettext('second')); 
		}
		return agoString
	},
	
	formatDatesOld: function(topElementId){
		if(topElementId == null){	
			var dates = document.getElementsByTagName("odate");	
		} else {
			var el = $(topElementId);
			var dates = el.getElementsByTagName("odate");	
		}
		for(i = 0; i<dates.length; i++){	
			// TODO: make it better ;-)
			var	timestamp = dates[i].innerHTML;
			var date = new Date();
			date.setTime(timestamp*1000);
			var dstring = date.toLocaleString();
			dates[i].innerHTML = dstring;
		}	
		
		
	},
	
	/**
	 * This is tricky. Loads desired url with parameters but the parameters
	 * are contained in the POST body.
	 */
	loadPage: function(url,parameters){
		// create a dummy form
		var form = document.createElement('form');
		for(p in parameters){
			var input = document.createElement('input');
			input.type="hidden";
			input["name"] = p;
			input.value=parameters[p];
			form.appendChild(input);
		}
		form.name="loadPageForm";
		form.action = url;
		form.method = 'post';
		form.display="none";
		form.target="_self";
		document.getElementsByTagName('body').item(0).appendChild(form);
		form.submit();
	},
	getCookie: function(cookieName){
		if (document.cookie.length>0){
			var c_start = document.cookie.indexOf(cookieName + "=");
		  	if (c_start!=-1){ 
			    c_start=c_start + cookieName.length+1 
			    var c_end=document.cookie.indexOf(";",c_start);
			    if (c_end==-1){
			    		c_end=document.cookie.length;
			    }
		    		return unescape(document.cookie.substring(c_start,c_end));
		    } 
		  }
		return null;
	},
	/**
	 * Filter all substrings of form e.g. [[olang en:sdadsd|pl:asdadad|]]
	 */
	olang: function(text){
		return text.replace(/\[\[olang (.*?)\|\]\]/g, function(str, p1, offset, st){
			var lang = OZONE.lang;
			var lr = new RegExp(lang+':([^\|]*)(\||\]\])');
			var res = str.match(lr);
			if(res){
				return res[1];
			}
		}); 
	}
	
}

OZONE.lang = "en"; // default language

OZONE.loc = {}
OZONE.loc.messages = {};

OZONE.loc.addMessages = function(mlist, lang){
	if(!OZONE.loc.messages[lang]){
		OZONE.loc.messages[lang] = {};
	}
	for(var i in mlist){
		OZONE.loc.messages[lang][i] = mlist[i];
	}
}


OZONE.loc.addMessage = function(mid, mtr, lang){
	if(!OZONE.loc.messages[lang]){
		OZONE.loc.messages[lang] = {};
	}
	OZONE.loc.messages[lang][mid] = mtr;
}



OZONE.loc.getMessage = function(mid, lang){
	if(OZONE.loc.messages[lang]){
		if(OZONE.loc.messages[lang][mid]){
			return OZONE.loc.messages[lang][mid];
		}
	}
	// fall back to default
	return mid;
}

ogettext = function(mid){
	return OZONE.loc.getMessage(mid, OZONE.lang);
}

OZONE.visuals = {
	cursorWait: function() {
		var body = document.getElementsByTagName("body")[0];
		YAHOO.util.Dom.addClass(body, "wait");
	},


	cursorClear: function() {
		var body = document.getElementsByTagName("body")[0];
		YAHOO.util.Dom.removeClass(body, "wait");
	},
	
	scrollTo: function(elementId, options){
		OZONE.visuals.scrollToCenter(elementId,options);
	
	},
	
	/**
	 * Vertically scrolls to the element in a way that the element is now in the CENTER
	 * of the page.
	 */
	scrollToCenter: function(element, options){
		
		var myEffect = new fx.ScrollCenter({duration: 200, transition: fx.sineOut});
		myEffect.scrollTo(element);
		if(options != null && options.blink==true){
			ofx34 = new fx.Opacity(element,{duration:150, transition: fx.circ});
			setTimeout("ofx34.custom(1,0.1)", 300);
			setTimeout("ofx34.custom(0.1,1);", 1000);
		}
		if(options != null && options.alterHref==true){
			// commented out because causes page jump...
		}
	},

	scrollOffsetY: function(){
		var y;
		if (self.pageYOffset) // all except Explorer
		{
			y = self.pageYOffset;
		}
		else if (document.documentElement && document.documentElement.scrollTop)
			// Explorer 6 Strict
		{
			y = document.documentElement.scrollTop;
		}
		else if (document.body) // all other Explorers
		{
			y = document.body.scrollTop;
		}
		return y;
	},
	
	bodyHeight: function(){
		var x,y;
			var test1 = document.body.scrollHeight;
			var test2 = document.body.offsetHeight
			if (test1 > test2) // all but Explorer Mac
			{
				return  document.body.scrollHeight;
			}
			else // Explorer Mac;
			     //would also work in Explorer 6 Strict, Mozilla and Safari
			{
				return document.body.offsetHeight;
			}
	},
	
	initScroll: function(){
		
		if(window.location.hash!= null && window.location.href!=''){
			var id = window.location.hash.replace(/#/, '');
			if(id!=null && id!='' && $(id)){
				OZONE.visuals.scrollTo(id, {blink:true});
			}
		}
	},
	
	/** TODO later.*/
	highlightText: function(root, text){
		// split the text by space (if any)
		if(text.indexOf(' ') != -1){
			var tarray = text.split(/ +/);
			for(var i=0;i<tarray.length; i++){
				if(!tarray[i].match(/^\-/)){
					OZONE.visuals.highlightText(root, tarray[i])
				}
			}
			return;
		}

		root = $(root);
		if(!root){return;}
		
		// recurrence first
		if (root.hasChildNodes){
			var chn = root.childNodes;
			for (var i=chn.length-1; i>=0;i--) {
				
				OZONE.visuals.highlightText(chn[i], text);
			}
		}
		if(root.nodeType == 3){ // text node
			// purify text a bit
			
			var reg = new RegExp(text, "gi");
			if(root.nodeValue.match(reg)){
				var contArray = (' '+root.nodeValue+' ').split(reg);
				p = root.parentNode;
				for(var i=0; i<contArray.length; i++){
					if(i!=0){
						var span = document.createElement('span');
						span.className="search-highlight";
						span.appendChild(document.createTextNode(text));
						p.insertBefore(span, root);
					}
					var z = document.createTextNode(contArray[i]);
					if(i != contArray.length-1){
						p.insertBefore(z,root);
					}else{
						p.replaceChild(z, root);
					}
				}
			}
		}
	}

}

OZONE.forms = {};
OZONE.forms.lengthLimiter = function(textElement, countElement, limit){
	this.textElement = $(textElement);
	this.countElement = $(countElement);
	this.limit = limit;
	
	
	YAHOO.util.Event.addListener(this.textElement, "keyup", this.keyListener, this, true);
	this.keyListener();
	
}
OZONE.forms.lengthLimiter.prototype.keyListener = function(e){
		// get number of characters...
		var chars = this.textElement.value.replace(/\r\n/, "\n").length;
		this.countElement.innerHTML = this.limit - chars;
		if(chars>this.limit){
			var scrollTop = this.textElement.scrollTop;
			this.textElement.value = this.textElement.value.substr(0,this.limit);
			this.textElement.scrollTop = scrollTop;
			chars = this.textElement.value.replace(/\r\n/, "\n").length;
			this.countElement .innerHTML = this.limit - chars;
		}
	}

OZONE.dom = {
	insertAfter: function(parentNode, node, referenceNode){
		if(referenceNode.nextSibling){
			parentNode.insertBefore(node, referenceNode.nextSibling);
		} else {
			parentNode.appendChild(node);
		}
	},
	
	onDomReady: function(f, el, doc){
		if(!doc){
			doc = document;
		}
		if(typeof doc.getElementsByTagName != 'undefined' 
				&& (doc.getElementsByTagName('body')[0] != null || doc.body != null)
				&& (typeof el != 'string' || $(el))){
			if(typeof f == 'function'){f();}
			else {OZONE.dom.onDomReady.fs[f].call();}
		}else{
			var fid;
			if(typeof f == 'function'){
				if(!OZONE.dom.onDomReady.fs){
					OZONE.dom.onDomReady.fs = new Array();
				}
				fid = OZONE.dom.onDomReady.fs.push(f) - 1;
			}else{
				fid = f;
			}
			
			var call = 'OZONE.dom.onDomReady('+fid;
			if(typeof el == 'string'){call+=',"'+el+'"';}
			call+=')';
			
			setTimeout(call, 200);
		}
	}
}

OZONE.request = {};

OZONE.init = function(){

}

//OZONE.init();
