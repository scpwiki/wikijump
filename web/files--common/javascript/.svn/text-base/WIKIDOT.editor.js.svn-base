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

var INSERT_NEWLINE = "\n";
var MATCH_NEWLINE = "\r?\n";

WIKIDOT.Editor = {
	editElementId: null,
	toolbarPanelId: null,
	ranger: null // a TextElementProxyUtil object
}

WIKIDOT.Editor.currentPos; //required by IE not to loose position when opening a wizard window


WIKIDOT.Editor.init = function(editElementId, toolbarPanelId){
	WIKIDOT.Editor.editElementId = editElementId;
	WIKIDOT.Editor.toolbarPanelId = toolbarPanelId;
	WIKIDOT.Editor.ranger = new TextElementProxyUtil(editElementId);
	YAHOO.util.Event.addListener(this.editElementId, "keypress", WIKIDOT.Editor.keyboardListener);
	YAHOO.util.Event.addListener(this.editElementId, "keydown", function(){WIKIDOT.Editor.lastKeyCode = null;});
	YAHOO.util.Event.addListener(this.editElementId, "keyup", WIKIDOT.Editor.codeAssist.listener);
	
	
	var durl;
	switch(OZONE.lang){
		case 'pl':
			durl = "/common--editor/dialogs-pl.html";
			break;
		default:
			durl = "/common--editor/dialogs.html";
	}
	
	//also read and add dialogs data
	YAHOO.util.Connect.asyncRequest('GET', durl , WIKIDOT.Editor.initCallback, null);
	
	// init newline character
	
	OZONE.loc.addMessage("cancel", "anuluj", "pl");
	OZONE.loc.addMessage("insert code", "wstaw kod", "pl");
	OZONE.loc.addMessage("Image wizard", "Magik wstawiania obrazu", "pl");
	OZONE.loc.addMessage("Table wizard", "Magik tabeli", "pl");
	
	
}

WIKIDOT.Editor.shutDown = function(){
	YAHOO.util.Event.removeListener(this.editElementId, "keypress", WIKIDOT.Editor.keyboardListener);
	YAHOO.util.Event.removeListener(this.editElementId, "keyup", WIKIDOT.Editor.codeAssist.listener);
	WIKIDOT.Editor.ranger = null;
	WIKIDOT.Editor.toolbarPanelId  = null;
	WIKIDOT.Editor.editElementId = null;
}

WIKIDOT.Editor.initCallback = {
	success: function(o) {
		var content = o.responseText;
		var div = document.createElement('div');
		div.id = "wd-ed-dialogs";
		div.innerHTML = content;
		div.style.display = "none";
		
		var body = document.getElementsByTagName('body').item(0);
		body.appendChild(div);
		var etoolbar = $("wd-ed-toolbar");
		var panel = $(WIKIDOT.Editor.toolbarPanelId);
		if(panel){
			
			panel.innerHTML = OZONE.utils.olang(etoolbar.innerHTML);
			var as = panel.getElementsByTagName('a');
			OZONE.dialog.hovertip.makeTip(as, {style: {width: 'auto'}, delay: 200});
			WIKIDOT.page.fixers.fixMenu(panel);
		}
	}, 
  	failure: function(o) {alert("failure error code\n823468008623487666624")} 
}

/* buttons listeners */
WIKIDOT.Editor.buttons = {
	bold: function(e){
		WIKIDOT.Editor.utils.insertTags("**","**", "bold text", 
										WIKIDOT.Editor.utils.trimSelection);
	},
	italic: function(e){
		WIKIDOT.Editor.utils.insertTags("//","//", "italic text", 
										WIKIDOT.Editor.utils.trimSelection);
	},
	underline: function(e){
		WIKIDOT.Editor.utils.insertTags("__","__", "underline text", 
										WIKIDOT.Editor.utils.trimSelection);
	},
	strikethrough: function(e){
		WIKIDOT.Editor.utils.insertTags("--","--", "strikethrough text", 
										WIKIDOT.Editor.utils.trimSelection);
	},
	teletype: function(e){
		WIKIDOT.Editor.utils.insertTags("{{","}}", "teletype text", 
										WIKIDOT.Editor.utils.trimSelection);
	},
	superscript: function(e){
		WIKIDOT.Editor.utils.insertTags("^^","^^", "superscript", 
										WIKIDOT.Editor.utils.trimSelection);
	},
	subscript: function(e){
		WIKIDOT.Editor.utils.insertTags(",,",",,", "subscript", 
										WIKIDOT.Editor.utils.trimSelection);
	},
	raw: function(e){
		WIKIDOT.Editor.utils.insertTags("@@","@@", "raw text", 
										WIKIDOT.Editor.utils.trimSelection);
	},
	
	heading: function(e,level){
		var pluses='';
		for(var i=0; i<level; i++){pluses+="+";}
		WIKIDOT.Editor.utils.insertTags(pluses+" ","", "heading level "+level, 
										WIKIDOT.Editor.utils.trimSelection,
										WIKIDOT.Editor.utils.endWith2NewLine,
										WIKIDOT.Editor.utils.startWith2NewLine);
	},
	
	quote: function(e){
		WIKIDOT.Editor.utils.insertTags("> ","", "quoted text", 
										WIKIDOT.Editor.utils.processQuoteText,
										WIKIDOT.Editor.utils.endWithAtLeast1NewLine,
										WIKIDOT.Editor.utils.startWithAtLeast1NewLine);
	},
	hr: function(e){
		WIKIDOT.Editor.utils.insertText("------",WIKIDOT.Editor.utils.endWithAtLeast2NewLine,
										WIKIDOT.Editor.utils.startWithAtLeast2NewLine);
	},
	clearFloat: function(e,dir){
		var text = "~~~~";
		if(dir) text += dir;
		WIKIDOT.Editor.utils.insertText(text,WIKIDOT.Editor.utils.endWithAtLeast2NewLine,
										WIKIDOT.Editor.utils.startWithAtLeast2NewLine);
	},
	toc: function(e){
		WIKIDOT.Editor.utils.insertText("[[toc]]",WIKIDOT.Editor.utils.endWithAtLeast1NewLine,
										WIKIDOT.Editor.utils.startWithAtLeast1NewLine);
	},
	uri: function(e){
		WIKIDOT.Editor.utils.insertTags("[http://www.example.com ","]", "describe link", 
										WIKIDOT.Editor.utils.trimSelection);
	},
	pageLink: function(e){
		WIKIDOT.Editor.utils.insertTags("[[[","]]]", "page name", 
										WIKIDOT.Editor.utils.trimSelection);
	},
	math: function(e){
		WIKIDOT.Editor.utils.insertTags("[[math]]"+INSERT_NEWLINE,INSERT_NEWLINE+"[[/math]]", "insert LaTeX equation here", 
										WIKIDOT.Editor.utils.trimSelection,
										WIKIDOT.Editor.utils.endWithAtLeast2NewLine,
										WIKIDOT.Editor.utils.startWithAtLeast2NewLine);
	},
	numberedList: function(e){
		WIKIDOT.Editor.utils.insertTags("# ","", "list item", 
										WIKIDOT.Editor.utils.processNumberedList,
										WIKIDOT.Editor.utils.endWithAtLeast1NewLine,
										WIKIDOT.Editor.utils.startWithAtLeast1NewLine);
	},
	bulletedList: function(e){
		WIKIDOT.Editor.utils.insertTags("* ","", "list item", 
										WIKIDOT.Editor.utils.processBulletedList,
										WIKIDOT.Editor.utils.endWithAtLeast1NewLine,
										WIKIDOT.Editor.utils.startWithAtLeast1NewLine);
	},
	definitionList: function(e){
		WIKIDOT.Editor.utils.insertTags(": "," : definition", "item", 
										WIKIDOT.Editor.utils.processBulletedList,
										WIKIDOT.Editor.utils.endWithAtLeast1NewLine,
										WIKIDOT.Editor.utils.startWithAtLeast1NewLine);
	},
	
	increaseListIndent: function(e){
		WIKIDOT.Editor.utils.insertText('',WIKIDOT.Editor.utils.increaseListIndent);
		
	},
	decreaseListIndent: function(e){
		WIKIDOT.Editor.utils.insertText('',WIKIDOT.Editor.utils.decreaseListIndent);
		
	},
	footnote: function(e){
		WIKIDOT.Editor.utils.insertTags("[[footnote]] "," [[/footnote]]", "footnote text", 
										WIKIDOT.Editor.utils.trimSelection);
	},
	inlineMath: function(e){
		WIKIDOT.Editor.utils.insertTags("[[$ "," $]]", "insert LaTeX equation here", 
										WIKIDOT.Editor.utils.trimSelection);
	},
	code: function(e){
		WIKIDOT.Editor.utils.insertTags("[[code]]"+INSERT_NEWLINE,INSERT_NEWLINE+"[[/code]]", "insert the code here", 
										WIKIDOT.Editor.utils.trimSelection,
										WIKIDOT.Editor.utils.endWithAtLeast2NewLine,
										WIKIDOT.Editor.utils.startWithAtLeast2NewLine);
	},
	video: function(e){
		WIKIDOT.Editor.utils.insertTags("[[embedvideo]]"+INSERT_NEWLINE,INSERT_NEWLINE+"[[/embedvideo]]", "paste the html for the video here (Google Video, YouTube, Revver, Dailymotion)", 
										WIKIDOT.Editor.utils.trimSelection,
										WIKIDOT.Editor.utils.endWithAtLeast2NewLine,
										WIKIDOT.Editor.utils.startWithAtLeast2NewLine);
	},
	audio: function(e){
		WIKIDOT.Editor.utils.insertTags("[[embedaudio]]"+INSERT_NEWLINE,INSERT_NEWLINE+"[[/embedaudio]]", "paste the html for the audio here (odeo)", 
										WIKIDOT.Editor.utils.trimSelection,
										WIKIDOT.Editor.utils.endWithAtLeast2NewLine,
										WIKIDOT.Editor.utils.startWithAtLeast2NewLine);
	},
	image: function(e){
		WIKIDOT.Editor.utils.insertTags("[[image ","]]", "source", 
										WIKIDOT.Editor.utils.trimSelection);
	},
	div: function(e){
		WIKIDOT.Editor.utils.insertTags("[[div]]"+INSERT_NEWLINE,INSERT_NEWLINE+"[[/div]]", "block contents", 
										WIKIDOT.Editor.utils.trimSelection,
										WIKIDOT.Editor.utils.endWithAtLeast2NewLine,
										WIKIDOT.Editor.utils.startWithAtLeast2NewLine);
	},
	bibliography: function(e){
		WIKIDOT.Editor.utils.insertTags("[[bibliography]]"+INSERT_NEWLINE+": "," : full source reference"+INSERT_NEWLINE+"[[/bibliography]]", "label", 
										WIKIDOT.Editor.utils.trimSelection,
										WIKIDOT.Editor.utils.endWithAtLeast2NewLine,
										WIKIDOT.Editor.utils.startWithAtLeast2NewLine);
	},
	bibliographycitation: function(e){
		WIKIDOT.Editor.utils.insertTags("[((bibcite ","))]", "label", 
										WIKIDOT.Editor.utils.trimSelection);
	},
	
	imageWizard: function(e){
		WIKIDOT.Editor.currentPos = WIKIDOT.Editor.ranger.getSelectionRange()[0];
		// open a dialog...
		var d = new OZONE.dialogs.Dialog();
		d.style.width = "70%";
		d.title=ogettext("Image wizard");
		d.buttons = ["cancel", "insert code"];
		d.addButtonListener("cancel", d.close);
		d.addButtonListener("insert code", WIKIDOT.Editor.imageWizard.insertCode);
		d.content = $("wd-ed-imagewizard-dialog").innerHTML.replace(/\-template/g, "");
		d.show();
		WIKIDOT.Editor.imageWizard.updateSourceBlock();
		
	},
	tableWizard: function(e){
		WIKIDOT.Editor.currentPos = WIKIDOT.Editor.ranger.getSelectionRange()[0];
		// open a dialog...
		var d = new OZONE.dialogs.Dialog();
		d.title=ogettext("Table wizard");
		d.buttons = ["cancel", "insert code"];
		d.addButtonListener("cancel", d.close);
		d.addButtonListener("insert code", WIKIDOT.Editor.listeners.tableWizardInsert);
		d.content = $("wd-ed-tablewizard-dialog").innerHTML.replace(/\-template/g, '');
		d.show();
		
	},
	uriWizard: function(e){
		WIKIDOT.Editor.currentPos = WIKIDOT.Editor.ranger.getSelectionRange()[0];
		// open a dialog...
		var d = new OZONE.dialogs.Dialog();
		d.title=ogettext("URL link wizard");
		d.buttons = ["cancel", "insert code"];
		d.addButtonListener("cancel", d.close);
		d.addButtonListener("insert code", WIKIDOT.Editor.listeners.uriWizardInsert);
		d.content = $("wd-ed-uriwizard-dialog").innerHTML.replace(/\-template/g, '');
		d.show();
		
	},
	pageLinkWizard: function(e){
		WIKIDOT.Editor.currentPos = WIKIDOT.Editor.ranger.getSelectionRange()[0];
		var d = new OZONE.dialogs.Dialog();
		d.title=ogettext("Page link wizard");
		d.buttons = ["cancel", "insert code"];
		d.addButtonListener("cancel", d.close);
		d.addButtonListener("insert code", WIKIDOT.Editor.listeners.pageLinkWizardInsert);
		d.content=$("wd-ed-pagelinkwizard-dialog").innerHTML.replace(/\-template/g, "");
		d.show();
		// attach the autocomplete thing
		var myDataSource = new YAHOO.widget.DS_XHR("/quickmodule.php", ['pages', 'unix_name', 'title']); 
		myDataSource.scriptQueryParam="q";
		myDataSource.scriptQueryAppend = "s="+WIKIREQUEST.info.siteId+"&module=PageLookupQModule&title=yes";

		var myAutoComp = new YAHOO.widget.AutoComplete("wd-ed-pagelinkwizard-page","autocomplete3432", myDataSource);
		myAutoComp.formatResult = function(aResultItem, sQuery) { 
			var title = aResultItem[1];
			var unixName = aResultItem[0];
			if(unixName!= null){
				return '<div >'+unixName+'</div><div style="font-size: 85%;">('+title+')</div>';
			} else {
				return "";
			}
	
		}
		myAutoComp.minQueryLength = 2;
		myAutoComp.queryDelay = 0.5;
		myAutoComp.forceSelection = false;
		myAutoComp.autoHighlight = false;
		
	},
	codeWizard: function(e){
		WIKIDOT.Editor.currentPos = WIKIDOT.Editor.ranger.getSelectionRange()[0];
		// open a dialog...
		var d = new OZONE.dialogs.Dialog();
		d.title=ogettext("Code block wizard");
		d.buttons = ["cancel", "insert code"];
		d.addButtonListener("cancel", d.close);
		d.addButtonListener("insert code", WIKIDOT.Editor.listeners.codeWizardInsert);
		d.content = $("wd-ed-codewizard-dialog").innerHTML.replace(/\-template/g, "");
		d.show();
	},
	erefWizard: function(e){
		WIKIDOT.Editor.currentPos = WIKIDOT.Editor.ranger.getSelectionRange()[0];
		// open a dialog...
		var d = new OZONE.dialogs.Dialog();
		d.title=ogettext("Equation reference wizard");
		d.buttons = ["cancel", "insert code"];
		d.addButtonListener("cancel", d.close);
		d.addButtonListener("insert code", WIKIDOT.Editor.erefWizard.insertCode);
		d.content = $("wd-ed-erefwizard-dialog").innerHTML.replace(/\-template/g, "");
		d.show();
		// now find all the equations...
		var text = $(WIKIDOT.Editor.editElementId).value;
		var refs = text.match(/^\[\[math\s([a-zA-Z0-9]+)\]\](\r?\n.*)*?\r?\n\[\[\/math\]\]/mg);
		if(refs == null || refs.length == 0){
			var w = new OZONE.dialogs.ErrorDialog();
			w.content = "Sorry, no labelled equations found.";
			w.show();
			return;	
		}
		var inn='';
		if(refs.length==0){
			inn="no equations with labels available";
		}else{
			inn = '<select id="wd-ed-erefwizard-ref">'
			for(var i=0; i<refs.length;i++){
				var elabel = refs[i].replace(/\[\[math\s(.+?)\]\](\r*\n.*)*/,'$1');
				var epreview = refs[i].replace(/\[\[math[^\]]*\]\]((?:\r?\n.*)*?)\n\[\[\/math\]\]/,"$1");
				$("wd-ed-erefwizard-preview").innerHTML+='<div id="wd-ed-erefwizard-preview-'+elabel+'">'+epreview+'</div>';
				inn+='<option value="'+elabel+'">'+elabel+'</option>';
			}
			inn+='</select>';
		}
		$("wd-ed-erefwizard-options").innerHTML=inn;
		OZONE.dialog.factory.boxcontainer().centerContent();
		WIKIDOT.Editor.erefWizard.changeRef(null);
		YAHOO.util.Event.addListener("wd-ed-erefwizard-ref", "change", WIKIDOT.Editor.erefWizard.changeRef)
	}
}

WIKIDOT.Editor.erefWizard = {};
WIKIDOT.Editor.erefWizard.changeRef = function(e){
	var pdiv = $("wd-ed-erefwizard-preview");
	var prevs = pdiv.childNodes;
	for(var i = 0; i<prevs.length; i++){
		prevs[i].style.display="none";
	}
	$("wd-ed-erefwizard-preview-"+$("wd-ed-erefwizard-ref").value).style.display = "block";
}
WIKIDOT.Editor.erefWizard.insertCode = function(e){
	var elabel = $("wd-ed-erefwizard-ref").value;
	var out = '[[eref '+elabel+']]';
	if($("wd-ed-erefwizart-weq").checked == true){
		out = 'Eq.('+out+')';
	}
	WIKIDOT.Editor.ranger.setSelectionRange(WIKIDOT.Editor.currentPos, WIKIDOT.Editor.currentPos);
	WIKIDOT.Editor.utils.insertText(out);
	OZONE.dialog.cleanAll();
}


WIKIDOT.Editor.imageWizard = {};
WIKIDOT.Editor.imageWizard.updateSourceBlock = function(e){
	var source;
	$("wd-ed-imagewizard-byuri").style.display="none";
	$("wd-ed-imagewizard-byfile").style.display="none";
	$("wd-ed-imagewizard-byflickr").style.display="none";
	$("wd-ed-imagewizard-checkresult").innerHTML="";
	
	if($("342type1").checked == true){
		source = "uri";
		$("wd-ed-imagewizard-byuri").style.display="block";
		
	}else if($("342type2").checked == true){
		source = "file";
		$("wd-ed-imagewizard-byfile").style.display="block";
		
		WIKIDOT.Editor.imageWizard.updateAttachements();
	}else if($("342type3").checked == true){
		source = "flickr";
		$("wd-ed-imagewizard-byflickr").style.display="block";
		
	}
	WIKIDOT.Editor.imageWizard.source = source;
}

WIKIDOT.Editor.imageWizard.updateAttachements = function(){
	OZONE.ajax.requestModule("editor/ImageAttachedFileModule", {pageId: WIKIREQUEST.info.pageId}, WIKIDOT.Editor.imageWizard.updateAttachementsCallback);
}
WIKIDOT.Editor.imageWizard.updateAttachementsCallback = function(r){
	$("wd-ed-imagewizard-byfile-list").innerHTML = r.body;
	WIKIDOT.Editor.imageWizard.attachementSelect();
}

WIKIDOT.Editor.imageWizard.attachementSelect = function(e){
	var el =  $("wd-ed-imagewizard-byfile-filename");
	if(el){
		var filename = $("wd-ed-imagewizard-byfile-filename").value;
		var src = '/local--resized-images/'+WIKIREQUEST.info.requestPageName+'/'+filename+'/thumbnail.jpg';
		$("wd-ed-imagewizard-byfile-preview").src = src;
	}
}

WIKIDOT.Editor.imageWizard.checkFlickrImage = function(e){
	p = new Object();
	var res = $("wd-ed-imagewizard-checkresult");
	// check the type of source
	var input = $("wd-ed-imagewizard-flickr").value;
	// check if is a http or id number
	var flickrId = input.replace(/^http:\/\/(?:www\.)?flickr\.com\/.*?\/([0-9]+)(?:\/.*)?$/, "$1");
	var secret = null;
	// or photo's url
	if(input.match(/^http:\/\/static\.flickr\.com\/[0-9]+\/([0-9]+)_([0-9a-z]+).*$/)){
		flickrId = input.replace(/^http:\/\/static\.flickr\.com\/[0-9]+\/([0-9]+)_([0-9a-z]+).*$/, "$1");
		secret = input.replace(/^http:\/\/static\.flickr\.com\/[0-9]+\/([0-9]+)_([0-9a-z]+).*$/, "$2");
		p['secret'] = secret;
	}
	res.innerHTML = "checking image "+flickrId+"...";
	if(!flickrId.match(/^([0-9]+)$/)){
		res.innerHTML = '<p style="color: red">Not a valid input for the flickr.com image.</p>';
		return;
	}
	
	p['flickr_id'] = flickrId;
	
	OZONE.ajax.requestModule("editor/FlickrCheckModule", p, WIKIDOT.Editor.imageWizard.checkFlickrImageCallback);
	
	
}
WIKIDOT.Editor.imageWizard.checkFlickrImageCallback = function(r){
	var res = $("wd-ed-imagewizard-checkresult");
	res.innerHTML = r.body;
}

WIKIDOT.Editor.imageWizard.checkUriImage = function(e){
//	newwindow.titlebar=
	var input = $("wd-ed-imagewizard-uri").value;
	var newwindow = window.open("about:blank", "_blank",'location=no,menubar=no,titlebar=no,resizable=yes,scrollbars=yes,width=' + (screen.width*0.5) + ',height=' +
		(screen.height*0.5) + ',top='+ (screen.height*0.25) +',left='+(screen.width*0.25));
	newwindow.document.write('<html><head><title>Checking image...</title></head><body>' +
			'<div style="text-align: center"><p>	If you see the image below - that means the location of the image you have entered ' +
			'is ok.</p>	<img id="check-image" src="'+input+'" alt="image not available!"/>' +
			'<p><a href="javascript:;" onclick="window.close()">close this window</a></p></div></body></html>');
	var ii = newwindow.document.getElementById("check-image");
	YAHOO.util.Event.addListener(ii, "load", WIKIDOT.Editor.imageWizard.checkUriImageResize, newwindow);
}
WIKIDOT.Editor.imageWizard.checkUriImageResize = function(e, win){
	// resize the window
	var width = Math.min(this.width+200,screen.availWidth-100);
	var height = Math.min(this.height+200,screen.availHeight-100);
	var posleft = (screen.availWidth-width)*0.5;
	var postop = (screen.availHeight-height)*0.5;
	win.resizeTo( width, height);
	win.moveTo(posleft, postop);
}

WIKIDOT.Editor.imageWizard.insertCode = function(e){
	var sourceType = WIKIDOT.Editor.imageWizard.source;
	var source;
	if(sourceType == "uri"){
		source = $("wd-ed-imagewizard-uri").value;
	}else if(sourceType == "file"){
		source = $("wd-ed-imagewizard-byfile-filename").value;
	}else if(sourceType == "flickr"){
		var input = $("wd-ed-imagewizard-flickr").value;
		// check if is a http or id number
		var flickrId = input.replace(/^http:\/\/(?:www\.)?flickr\.com\/.*?\/([0-9]+)(?:\/.*)?$/, "$1");
		var secret = null;
		// or photo's url
		if(input.match(/^http:\/\/static\.flickr\.com\/[0-9]+\/([0-9]+)_([0-9a-z]+).*$/)){
			flickrId = input.replace(/^http:\/\/static\.flickr\.com\/[0-9]+\/([0-9]+)_([0-9a-z]+).*$/, "$1");
			secret = input.replace(/^http:\/\/static\.flickr\.com\/[0-9]+\/([0-9]+)_([0-9a-z]+).*$/, "$2");
		}
		if(!flickrId.match(/^([0-9]+)$/)){
			var res = $("wd-ed-imagewizard-checkresult");
			res.innerHTML = '<p style="color: red">Not a valid input for the flickr.com image.</p>';
			return;
		}
		source = 'flickr:'+flickrId;
		if(secret){ source+='_'+secret;}
	
	}
	
	// check if size
	var size ='';
	var el = $("wd-ed-imagewizard-size");
	if(el){
		size = el.value;
	}
	
	if(size !=''){
		size=' size="'+size+'"';
	}
	
	var position = $("wd-ed-imagewizard-position").value.replace(/l/,'<').replace(/r/, '>').replace(/c/,'=');
	var code = '[['+position+'image '+source+size+']]';
	WIKIDOT.Editor.ranger.setSelectionRange(WIKIDOT.Editor.currentPos, WIKIDOT.Editor.currentPos);
	WIKIDOT.Editor.utils.insertText(code);
	OZONE.dialog.cleanAll();
}
/**
 * Mainly wizard button listeners...
 */
WIKIDOT.Editor.listeners = {
	tableWizardInsert: function(e){
		
		var rows = $("wd-ed-tablewizard-rows").value;
		var columns = $("wd-ed-tablewizard-columns").value;
		var headers = $("wd-ed-tablewizard-headers").checked;
		
		// prepare code to be inserted
		var out='';
		for(var i = 0; i<rows; i++){
			out+=INSERT_NEWLINE+'||';
			for(var j=0; j<columns; j++){
				if(i==0 && headers){ 
					out+="~ header ||";
				}else {
					out+=" cell-content ||";
				}
			}
				
		}

		// insert it!
		WIKIDOT.Editor.ranger.setSelectionRange(WIKIDOT.Editor.currentPos, WIKIDOT.Editor.currentPos);
		
		WIKIDOT.Editor.utils.insertText(out,WIKIDOT.Editor.utils.endWithAtLeast1NewLine,
										WIKIDOT.Editor.utils.startWithAtLeast2NewLine);

		OZONE.dialog.cleanAll();
	},
	uriWizardInsert: function(e){
		var uri = $("wd-ed-uriwizard-uri").value;
		var anchor = $("wd-ed-uriwizard-anchor").value;
		var newwindow = $("wd-ed-uriwizard-newwindow").checked;
		
		var out='';
		if(anchor == null || anchor == ''){
			if(newwindow){
				out += '*';
			}
			out += uri;
		}else{
			out = '[';
			if(newwindow){
				out += '*';
			}
			out += uri+' '+anchor+']';
		}
		WIKIDOT.Editor.ranger.setSelectionRange(WIKIDOT.Editor.currentPos, WIKIDOT.Editor.currentPos);
		WIKIDOT.Editor.utils.insertText(out);
		OZONE.dialog.cleanAll();
	},
	pageLinkWizardInsert: function(e){
		var pageName = $("wd-ed-pagelinkwizard-page").value;
		var anchor = $("wd-ed-pagelinkwizard-anchor").value;
		
		var out = '[[['+pageName;
		if(anchor != null && anchor != ''){
			out += ' |' +anchor; 
		}
		out += ']]]';
		WIKIDOT.Editor.ranger.setSelectionRange(WIKIDOT.Editor.currentPos, WIKIDOT.Editor.currentPos);
		WIKIDOT.Editor.utils.insertText(out);
		OZONE.dialog.cleanAll();	
	},
	
	codeWizardInsert: function(e){
		var type = $("wd-ed-codewizard-type").value;
		var openTag = '[[code';
		if(type != ''){
			openTag += ' type="'+type+'"';
		}
		openTag += "]]"+INSERT_NEWLINE;
		var closeTag = INSERT_NEWLINE+"[[/code]]";
	
		WIKIDOT.Editor.ranger.setSelectionRange(WIKIDOT.Editor.currentPos, WIKIDOT.Editor.currentPos);
		
		WIKIDOT.Editor.utils.insertTags(openTag,closeTag, "insert the code here", 
										WIKIDOT.Editor.utils.trimSelection,
										WIKIDOT.Editor.utils.endWithAtLeast2NewLine,
										WIKIDOT.Editor.utils.startWithAtLeast2NewLine);

		OZONE.dialog.cleanAll();
	}
}


WIKIDOT.Editor.keyboardListener = function(e){
	WIKIDOT.Editor.lastKeyCode = null;
	var keyCode = YAHOO.util.Event.getCharCode(e);
	WIKIDOT.Editor.lastKeyCode = keyCode;
//		// trigger codeAssist
	var key='';
	if(e.ctrlKey == true) key += "ctrl+";
	if(e.altKey == true) key += "alt+";
	key += String.fromCharCode(keyCode);
	if($("editdebug")) $("editdebug").innerHTML = keyCode;
	var listener = WIKIDOT.Editor.keys[key];
	if(!listener) {listener = WIKIDOT.Editor.keyCodes[keyCode];}
	if(listener){
		YAHOO.util.Event.preventDefault(e);
		listener.call(null, e); 
	}

}
WIKIDOT.Editor.codeAssist = {};
WIKIDOT.Editor.codeAssist.listener = function(e){
	var keyCode = WIKIDOT.Editor.lastKeyCode;
	if(keyCode != 13){
		return;
	}
	
	// need to insert the "\n" manually here and stop event propagation
	
	
	// perform a number of checks if one should insert anything interesting...
	
	// check for a list item first
	
	WIKIDOT.Editor.utils.insertText("",WIKIDOT.Editor.codeAssist.rules.listEnd);//,
	WIKIDOT.Editor.utils.insertText("",WIKIDOT.Editor.codeAssist.rules.list);//,
	WIKIDOT.Editor.utils.insertText("",WIKIDOT.Editor.codeAssist.rules.listNested);//,

	WIKIDOT.Editor.codeAssist.rules.completeBlock();
	WIKIDOT.Editor.utils.insertText("",WIKIDOT.Editor.codeAssist.rules.definitionList);//,
	WIKIDOT.Editor.utils.insertText("",WIKIDOT.Editor.codeAssist.rules.keepIndent);
	WIKIDOT.Editor.utils.insertText("",WIKIDOT.Editor.codeAssist.rules.indentEnd);
	 
}
WIKIDOT.Editor.codeAssist.rules = {}

WIKIDOT.Editor.codeAssist.rules.list = function(text){
	/* check if previous line has anything to do with lists, i.e.
	 * 1. check if previous line starts with #|*
	 */
	text = text.replace(/(\r?\n([\*#])\s.*?\r?\n)$/, "$1$2 ");
	return text;
}
WIKIDOT.Editor.codeAssist.rules.definitionList = function(text){
	text = text.replace(/(\r?\n:\s.+?\s:.*\r?\n)$/, "$1: ");
	return text;
}
WIKIDOT.Editor.codeAssist.rules.listNested = function(text){
	/* this is different from list because requires one more line before
	 * check if previous line has anything to do with lists, i.e.
	 * 1. check if previous line starts with #|*
	 * 
	 */
	text = text.replace(/(\r?\n *[\*#]\s.+\r?\n( *)([\*#])\s.*?\r?\n)$/, "$1$2$3 ");
	return text;
}
WIKIDOT.Editor.codeAssist.rules.listEnd = function(text){
	/* after "double enter" remove list markup */
	text = text.replace(/(\r?\n\s*[\*#:]\s.*?\r?\n)\s*[\*#:]\s\r?\n$/, "$1"+INSERT_NEWLINE);
	return text;
}
WIKIDOT.Editor.codeAssist.rules.keepIndent = function(text){
	/* keeps the identation from the previous line */
	text = text.replace(/(\r?\n(\t+).+\r?\n)$/, "$1$2");
	return text;
}
WIKIDOT.Editor.codeAssist.rules.indentEnd = function(text){
	/* keeps the identation from the previous line */
	text = text.replace(/(\r?\n(\t+)\r?\n)$/, INSERT_NEWLINE+INSERT_NEWLINE);
	return text;
}
/**
 * Checks if the previous line contains any block start mark and stores
 * the name of the block in a variable
 */
WIKIDOT.Editor.codeAssist.rules.completeBlock = function(){
	
	var field = $(WIKIDOT.Editor.editElementId);
	var scrollTop = field.scrollTop;
	var ranger = WIKIDOT.Editor.ranger;
	var range = ranger.getSelectionRange();
	var before = field.value.substring(0,range[1]);
	var after = field.value.substring(range[1], field.value.length);
	var beforeOrigLength = before.length;
	before = before.replace(/(\[\[(div|code|embedvideo|math|embed)(?:\s[^\]]*?)?\]\]\r?\n)$/,"$1"+INSERT_NEWLINE+"[[/$2]]");
	field.value = before+after;
	var cursorPos =  beforeOrigLength;
	ranger.setSelectionRange(cursorPos, cursorPos);
	field.scrollTop = scrollTop;
}
WIKIDOT.Editor.codeAssist.rules.completeBlockPost = function(text){
	
}

WIKIDOT.Editor.keys = new Object();
WIKIDOT.Editor.keys["ctrl+b"] = WIKIDOT.Editor.buttons.bold;
WIKIDOT.Editor.keys["ctrl+i"] = WIKIDOT.Editor.buttons.italic;
WIKIDOT.Editor.keys["ctrl+u"] = WIKIDOT.Editor.buttons.underline;

WIKIDOT.Editor.keyCodes = new Object();
WIKIDOT.Editor.keyCodes[9] = function(e){
			WIKIDOT.Editor.utils.insertText("\t");
			YAHOO.util.Event.stopEvent(e);
		}

WIKIDOT.Editor.utils = {};
	
WIKIDOT.Editor.utils.insertTags = function(openTag, closeTag, sampleText, 
										processSelection, processBefore, processAfter, dontSelectSampleText){
	
	var myField = $(WIKIDOT.Editor.editElementId);
	myField.focus();
	var ranger = WIKIDOT.Editor.ranger;
	ranger.trimSelection();
	var range = ranger.getSelectionRange();
	
	var scrollTop = myField.scrollTop;
	
	var beforeText = myField.value.substring(0, range[0]);
	if(processBefore){
		beforeText = processBefore.call(null, beforeText);
	}
	var afterText =  myField.value.substring(range[1], myField.value.length);
	if(processAfter){
		afterText = processAfter.call(null, afterText);
	}	
	
	if (range[0] != range[1]) {
		var selectionText = myField.value.substring(range[0], range[1]);
		if(processSelection){
			selectionText = processSelection.call(null, selectionText);
		}
			
		myField.value = beforeText
		              + openTag
		              + selectionText
		              + closeTag
		              + afterText;
		var cursorPos = myField.value.length - afterText.length;
		ranger.setSelectionRange(cursorPos, cursorPos);
	}else {
		myField.value = beforeText
		              + openTag
		              + sampleText 
		              + closeTag
		              + afterText;
		if(!dontSelectSampleText){
			var startPos = beforeText.length + openTag.length
			var endPos = startPos + sampleText.length
			ranger.setSelectionRange(startPos, endPos);	
		} else {
			// just position the cursor after the text
			var cursorPos = myField.value.length - afterText.length;
			ranger.setSelectionRange(cursorPos, cursorPos);
		}
	}
	myField.focus();
	myField.scrollTop = scrollTop;
		
	
	
	
	
}

WIKIDOT.Editor.utils.insertText = function(text, processBefore, processAfter){
	
	WIKIDOT.Editor.utils.insertTags('','', text, 
										null, processBefore, processAfter, true);
	return;
	//WIKIDOT.Editor.utils.insertTags("", "", text, 
	var myField = $(WIKIDOT.Editor.editElementId);
	var ranger = WIKIDOT.Editor.ranger;
	var range = ranger.getSelectionRange();
	var scrollTop = myField.scrollTop;
	
	var beforeText = myField.value.substring(0, range[0]);
	if(processBefore){
		beforeText = processBefore.call(null, beforeText);
	}
	var afterText =  myField.value.substring(range[1], myField.value.length);
	if(processAfter){
		afterText = processAfter.call(null, afterText);
	}	
	
	myField.value = beforeText
	              + text
	              + afterText;
	var cursorPos = beforeText.length;//myField.value.length - afterText.length;
	ranger.setSelectionRange(cursorPos, cursorPos);
	
	myField.focus();
	myField.scrollTop = scrollTop;
	
	
	
}

WIKIDOT.Editor.utils.trimSelection = function(string){
	return string.replace(/^\s+/, '').replace(/\s+$/, '');
}
/** 
 * Checks if a string ends with a newline and of no, adds it.
 */
WIKIDOT.Editor.utils.endWithNewLine = function(string){
		return string.replace(/[\s\r\n]+$/, '')+INSERT_NEWLINE;
}
WIKIDOT.Editor.utils.endWithAtLeast1NewLine = function(string){
		return string.replace(/\r?\n$/, '')+INSERT_NEWLINE;
}
WIKIDOT.Editor.utils.startWithNewLine = function(string){
	return INSERT_NEWLINE+string.replace(/^[\s\r\n]+/, '');
}
WIKIDOT.Editor.utils.startWithAtLeast1NewLine = function(string){
	if(string.length == 0) { return string;}
	return INSERT_NEWLINE+string.replace(/^\r?\n/, '');
}

WIKIDOT.Editor.utils.startWithAtLeast2NewLine = function(string){
	if(string.length == 0) { return string;}
	return INSERT_NEWLINE+INSERT_NEWLINE+string.replace(/^\r?\n(\s*\r?\n)?/, '');
}
WIKIDOT.Editor.utils.endWithAtLeast2NewLine = function(string){
	if(string.length == 0) { return string;}
	return string.replace(/(\r?\n\s*)?\r?\n$/, '')+INSERT_NEWLINE+INSERT_NEWLINE;
}

WIKIDOT.Editor.utils.endWith2NewLine = function(string){
	if(string.length == 0) { return string;}
	return string.replace(/[\s\r\n]+$/, '')+INSERT_NEWLINE+INSERT_NEWLINE;
}

WIKIDOT.Editor.utils.startWith2NewLine = function(string){
	return INSERT_NEWLINE+INSERT_NEWLINE+string.replace(/^[\s\r\n]+/, '');
}

WIKIDOT.Editor.utils.processQuoteText = function(string){
	var string = string.replace(/^\s+/, '').replace(/\s+$/, '');
	string = string.replace(/\r?\n/g, INSERT_NEWLINE+"> ")
	return string;
}
WIKIDOT.Editor.utils.processNumberedList = function(string){
	var string = string.replace(/^\s+/, '').replace(/\s+$/, '');
	string = string.replace(/\r?\n/g, INSERT_NEWLINE+"# ")
	return string;
}
WIKIDOT.Editor.utils.processBulletedList = function(string){
	var string = string.replace(/^\s+/, '').replace(/\s+$/, '');
	string = string.replace(/\r?\n/g, INSERT_NEWLINE+"* ")
	return string;
}
WIKIDOT.Editor.utils.increaseListIndent = function(text){
	//check if not "overnested"
	if(text.match(/\r?\n(\s*)[\*#].*\r?\n(\1)\s+[\*#].*$/)){
		return text;
	}
	return text.replace(/(\r?\n\s*[\*#].*)(\r?\n\s*)([\*#].*)$/, "$1$2 $3");
}
WIKIDOT.Editor.utils.decreaseListIndent = function(text){
	return text.replace(/(\r?\n\s*) ([\*#].*)$/, "$1$2");
}

//alert("test '"+WIKIDOT.Editor.utils.endWithNewLine("abd\n   ")+"'")
//alert(' test"'+WIKIDOT.Editor.utils.startWithNewLine("adasdad")+'"')

/*
A piece of code that might be useful for IE:

var element = document.getElementById( 'my_textarea' );
if( document.selection ){
// The current selection
var range = document.selection.createRange();
// We'll use this as a 'dummy'
var stored_range = range.duplicate();
// Select all text
stored_range.moveToElementText( element );
// Now move 'dummy' end point to end point of original range
stored_range.setEndPoint( 'EndToEnd', range );
// Now we can calculate start and end points
element.selectionStart = stored_range.text.length - range.text.length;
element.selectionEnd = element.selectionStart + range.text.length;
}

from: http://the-stickman.com/web-development/javascript/finding-selection-cursor-%20position-in-a-textarea-in-internet-explorer/
* 
 */

/**
 * Abstraction proxy for handling textarea range, cursor position etc.
 */
TextElementProxyUtil = function(fieldId){
	this.field = $(fieldId);
	/* determine type of the browser. IE vs Gecko/Opera vs the rest.
	 */
	this.detectBrowser();
}

TextElementProxyUtil.prototype.detectBrowser = function(){

	if(this.field.selectionStart || this.field.selectionStart == 0){
		this.browserType = "gecko";
	} else {
		this.field.focus();
		if(document.selection.createRange){
			this.browserType = "ie";
			// also change newline character
			INSERT_NEWLINE = "\r\n";
		}
	}
	
}

TextElementProxyUtil.prototype.getCursorPosition = function(){
	var range = this.getSelectionRange();
	return range[1]; // end of the selection should be considered cursor position
}
/**
 * Return selection range of the text field as an 2-element array.
 */
TextElementProxyUtil.prototype.getSelectionRange = function(){
	var startPos;
	var endPos;
	this.field.focus();
	if(this.browserType == "gecko"){
		startPos = this.field.selectionStart;
		endPos = this.field.selectionEnd;
	}		
	if(this.browserType == "ie"){
		if( document.selection ){
 			
 			var range = document.selection.createRange();
			var storedRange = range.duplicate();
			storedRange.moveToElementText(this.field);
			storedRange.setEndPoint( 'StartToStart', range );
			startPos = this.field.value.length - storedRange.text.length;
			endPos = startPos+range.text.length;
			
		}
	}
	if(this.browserType == "rest"){
			
	}
	this.field.focus();
	return [startPos, endPos];
}

TextElementProxyUtil.prototype.setSelectionRange = function(startPos, endPos){
	this.field.focus();
	if(this.browserType == "gecko"){
		this.field.setSelectionRange(startPos, endPos);
	}		
	if(this.browserType == "ie"){
		// fix position: Windows based "new lines" (\r\n) are counted as 2 characters, 
		// but not when it comes to positioning the cursor!!!
		var beforeText = this.field.value.substring(0, startPos);
		var selText = this.field.value.substring(startPos, endPos);
		startPos = beforeText.replace(/\r\n/g, "\n").length;
		endPos = startPos + selText.replace(/\r\n/g, "\n").length;
		var range = this.field.createTextRange();
	    range.collapse(true);
	    range.moveEnd('character', endPos);
	    range.moveStart('character', startPos);
	    range.select(); 
	}
	if(this.browserType == "rest"){
		
	}
	this.field.focus();
}

/**
 * Trims the selection to remove whitespaces from the begining or end of the selection.
 */
TextElementProxyUtil.prototype.trimSelection = function(){
	var range = this.getSelectionRange();
	var selectionText = this.field.value.substring(range[0], range[1]);
	var trimLeft = selectionText.length - selectionText.replace(/^\s+/,"").length;
	var trimRight = selectionText.length - selectionText.replace(/\s+$/,"").length;
	this.setSelectionRange(range[0]+trimLeft, range[1]-trimRight);
	
}
