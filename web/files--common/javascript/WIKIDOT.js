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

var WIKIDOT = {};
var WIKIDOT = {
	page: function(){},
	modules: function(){},
	utils: function(){},
	vars: function(){}
}
 


WIKIDOT.utils = {
	changeTextareaRowNo: function(textarea, numModifier){
		var ta = $(textarea);
		if((numModifier<0 && ta.rows+numModifier>=5) || (numModifier>0 && ta.rows+numModifier<=50)) {
			ta.rows = ta.rows + numModifier;
		}
	}
}

WIKIDOT.visuals = {
	/**
	 * Creates a place in the center of the screen to place a message.
	 */
	showCenterMessage: function(text){
		var body = document.getElementsByTagName('body').item(0);
		
		sDiv = document.createElement('div');
		sDiv.id = 'center-message-shader';
		bodyHeigh = body.offsetHeight+50;
		viewportHeight = YAHOO.util.Dom.getClientHeight();
		height = Math.max(bodyHeigh,viewportHeight);
		sDiv.style.height = height+"px";
		
		// now create the table wrapper environment for the message itself!
		mTable = document.createElement('table');
		mTable.id = "center-message-wrapper";
		mRow = document.createElement("tr");
		mTd = document.createElement("td");
		mRow.appendChild(mTd);
		mTable.appendChild(mRow);
		mDiv = document.createElement('div');
		mDiv.id="center-message";
		mTd.appendChild(mDiv);
		
		mDiv.innerHTML=text;
		
		sDiv.style.visibility = "hidden";
		mTable.style.visibility = "hidden";
		body.appendChild(sDiv);
		body.appendChild(mTable);
		
		YAHOO.util.Dom.setY("center-message-wrapper",OZONE.visuals.scrollOffsetY());
		mTable.style.height = viewportHeight+"px";
	
		ofx = new fx.Opacity("center-message",{duration:100});
		ofx.setOpacity(0);
		
		sDiv.style.visibility = "visible";
		mTable.style.visibility = "visible";
		ofx.custom(0,1);
	},
	
	changeCenterMessage: function(text){
		ofx = new fx.Opacity("center-message",{duration:100});
		ofx.custom(1,0);
		setTimeout('OZONE.utils.setInnerHTMLContent("center-message", "'+text+'");ofx.custom(0,1)', 200);
	},
	
	hideCenterMessage: function(){
		var body = document.getElementsByTagName('body').item(0);
		sDiv = document.getElementById("center-message-shader");
		mTable = document.getElementById("center-message-wrapper");
		if(sDiv != null){
			body.removeChild(sDiv);
			body.removeChild(mTable);
		}	
	 
	}
}

WIKIDOT.utils.formatDates = function(topElementId){
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
	
	
}
/**
 * Displays simple error messages as dialogs.
 */
WIKIDOT.utils.handleError = function(r){
	if(r.status != 'ok'){
		var w = new OZONE.dialogs.ErrorDialog();
		if(r.status == 'no_permission'){
			w.title = ogettext('Permission error');
		}
		w.content = '<h1>'+ogettext('Oooops!')+'</h1><p>'+r.message+'</p>'; 
		w.show();	
		return false;
	}else{
		return true;
	}
}

WIKIDOT.render = {}; 
WIKIDOT.render.printuser = function(userId, userName, wImage){
	var link = 'href="javascript:;" onclick="WIKIDOT.page.listeners.userInfo('+userId+')"';
	var out='<span class="printuser">';
	if(wImage==true){
		out += '<a '+link+' ><img class="small" src="/common--images/avatars/'+Math.floor(userId/1000)+'/'+userId+'/a16.png" ' +
			' alt="" style="background-image:url(/userkarma.php?u=' + userId + ')"/>';
	}
	out += '<a '+link+'>'+userName+'</a></span>';
	return out;
}

WIKIDOT.render.fixAvatarHover = function(root){
	var users = YAHOO.util.Dom.getElementsByClassName("printuser avatarhover", "span", root);
	for(var i = 0; i<users.length; i++){
		var a = users[i].getElementsByTagName("a")[0];
		if(a.getElementsByTagName("img").length == 1){
			// ok, has image, now attach hover listener!
			YAHOO.util.Event.addListener(a, "mouseover", WIKIDOT.render.fixAvatarHover.showHover);
		}
		
		
	}
	
}

WIKIDOT.render.fixAvatarHover.showHover = function(e){
	if($("avatar-hover-container") == null){
		var ac = document.createElement('div');
		ac.style.visibility="hidden";
		ac.style.position="absolute";
		ac.style.width = "100%";
		document.getElementsByTagName("body")[0].appendChild(ac);
	}else{
		ac = $("avatar-hover-container");
	}
	
	if(this.hoverAvatar == null){
		// make the hover avatar element
		var img0 = this.getElementsByTagName("img")[0];
		var newsrc = img0.src.replace(/a16\.png$/, 'a48.png');
		// now create a new a element
		
		var a = document.createElement("a");
		var img = document.createElement("img");
		img.src = newsrc;
		a.className = "avatar-hover";
		a.style.position = "absolute";
		a.style.display="none";
		a.href = this.href;
		a.onclick = this.onclick;
		
		var d = document.createElement("div");
		
		d.appendChild(img);
		
		
		a.appendChild(d);
		
		
		YAHOO.util.Dom.generateId(a);
		
		this.hoverAvatar = a;
		ac.appendChild(a);
		YAHOO.util.Event.addListener(a, "mouseout", WIKIDOT.render.fixAvatarHover.hideHover);
		YAHOO.util.Event.addListener(a, "mousemove", WIKIDOT.render.fixAvatarHover.mousemove);
		var eff = new fx.Opacity(a, {duration: 200});
		this.hoverAvatarEffect = eff;
		eff.setOpacity(0);
	}
	var ha = this.hoverAvatar
	//position the hover!
	var cx,cy;
	cx = YAHOO.util.Dom.getX(this) + 8 + 8; /* 8 is half of avatar width and 8 is karma indicator width */
	cy = YAHOO.util.Dom.getY(this) + 8;
	ha.style.display="block";
	var img = ha.getElementsByTagName('img')[0];
	var ih, iw;
	var lbind = false;
	if(img.height == 0){
		ih = 48;
		iw = 48;
		lbind=true;
		YAHOO.util.Event.addListener(img, "load", function(event, ha){
			var img = this;
			YAHOO.util.Dom.setXY(ha, [cx - (img.width/2 +8), cy - (img.height/2+8)]);
		}, ha);
	}else{
		ih=img.height;
		iw=img.width;
	}
	YAHOO.util.Dom.setXY(ha, [cx - (iw/2 +8), cy - (ih/2+8)]);
	
	if(ha.style.opacity == 0){
		this.hoverAvatarEffect.custom(0,1);
	}
	
	// set timer
	ha.lastAccess = (new Date()).getTime();
	setTimeout('WIKIDOT.render.fixAvatarHover.mousemove.autoHide("'+ha.id+'")', 1000);
	
}

WIKIDOT.render.fixAvatarHover.hideHover= function(e){
	var rt = YAHOO.util.Event.getRelatedTarget(e);
	if(!YAHOO.util.Dom.isAncestor(this, rt) && rt!=this){
		this.style.display = "none";	
		this.style.visibility = "hidden";
		this.style.opacity = 0;
	}
}

WIKIDOT.render.fixAvatarHover.mousemove = function(e){
	this.lastAccess = (new Date()).getTime();	

}

WIKIDOT.render.fixAvatarHover.mousemove.autoHide= function(hoverId){
	var a = $(hoverId);
	var now = (new Date()).getTime();
	if(a.lastAccess +3000 < now){
		a.style.display = "none";	
		a.style.visibility = "hidden";
		a.style.opacity = 0;
	}else{
		// check again	
		setTimeout('WIKIDOT.render.fixAvatarHover.mousemove.autoHide("'+a.id+'")', 1000);
	
	}
		
}
