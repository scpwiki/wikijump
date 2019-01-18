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

WIKIDOT.modules.LoginModule = {};

WIKIDOT.modules.LoginModule.listeners = {
	loginClick: function(e){
		YAHOO.util.Event.stopEvent(e);
		
		var p = OZONE.utils.formToArray("login-form");
		// pre-check:
		var welcome = OZONE.utils.getCookie('welcome');
		if((welcome == null && p['name'] == '') || p['password'] == ''){
			var message="Please fill the login form.";
			$('loginerror').innerHTML = message;
			//$("login-head").style.display = "none";
			$('loginerror').style.display="block";
			return false;
		}
		if(welcome){
			p['welcome'] = welcome;
			
		}
		
		
		p.action = "Login2Action";
		p.event = "login";
		OZONE.ajax.requestModule(null, p, WIKIDOT.modules.LoginModule.callbacks.loginClick);
	},
	
	switchUser: function(e){
		setCookie('welcome', null, -100000, '/', '.'+URL_DOMAIN);
		setCookie('welcome', null, -100000, '/');
		window.location.reload();
	},
	
	cancel: function(e){
		var url = getQueryString('origUrl', 'http://'+URL_HOST); 
		window.location.href = url;
	},
	
	namePress: function(e){
		var chcode = YAHOO.util.Event.getCharCode(e);
		if((chcode == 13 || chcode == 9) && $('login-form-name').value.length>0 ){
			YAHOO.util.Event.preventDefault(e);
			$('login-form-password').focus();
		}
	}

}

WIKIDOT.modules.LoginModule.callbacks = {
	loginClick: function(r){
		if(r.status == 'login_invalid'){
			$("loginerror").innerHTML=r.message;
			$("loginerror").style.display = "block";
			return;
		}
		
		if(!WIKIDOT.utils.handleError(r)) {return;}
		var w = new OZONE.dialogs.WaitBox();
		w.content = "Logging in...";
		w.show();
		var originalUrl = r.originalUrl;
		if(originalUrl){
			window.location.href=originalUrl;
		}else{
			window.location.href='http://'+window.location.host;
		}
	},
	
	cancel: function(r){
		window.location.href='http://'+window.location.host;
	}
}

WIKIDOT.modules.LoginModule.init = function(){
	if($('login-form-name')){
		$('login-form-name').focus();
		YAHOO.util.Event.addListener($('login-form-name'), 'keypress', WIKIDOT.modules.LoginModule.listeners.namePress);
	}else{
		$('login-form-password').focus();
	}
	
	OZONE.dom.onDomReady(function(){		
		// change links to http://...
		var els = document.getElementsByTagName('a');
		for(var i=0; i<els.length;i++){
			els[i].href = els[i].href.replace(/^https/, 'http');
		}
	}, "dummy-ondomready-block");
}

setTimeout(function(){WIKIDOT.modules.LoginModule.init();}, 100);

function getQueryString(key, default_)
{
  if (default_==null) default_="";
  key = key.replace(/[\[]/,"\\\[").replace(/[\]]/,"\\\]");
  var regex = new RegExp("[\\?&]"+key+"=([^&#]*)");
  var qs = regex.exec(window.location.href);
  if(qs == null)
    return default_;
  else
    return decodeURIComponent(qs[1]);
} 


function setCookie( name, value, expires, path, domain, secure) {
	var today = new Date();
	today.setTime( today.getTime() );
	if ( expires ) {
		expires = expires * 1000 * 60 * 60 * 24;
	}
	var expires_date = new Date( today.getTime() + (expires) );
	
	var ck = name+'='+escape( value ) +
	
	( ( expires ) ? ';expires='+expires_date.toGMTString() : '' ) + //expires.toGMTString()
	( ( path ) ? ';path=' + path : '' ) +
	( ( domain ) ? ';domain=' + domain : '' ) +
	( ( secure ) ? ';secure' : '' );
	//alert(ck);
	document.cookie = ck;
}

