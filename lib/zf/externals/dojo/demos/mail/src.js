dojo.require("dojo.parser");
dojo.require("dojo.data.ItemFileWriteStore");

dojo.require("dijit.dijit");
dojo.require("dijit.Declaration");
dojo.require("dijit.form.Button");
dojo.require("dijit.Menu");
dojo.require("dijit.Tree");
dojo.require("dijit.Tooltip");
dojo.require("dijit.Dialog");
dojo.require("dijit.Toolbar");
dojo.require("dijit._Calendar");
dojo.require("dijit.ColorPalette");
dojo.require("dijit.Editor");
dojo.require("dijit._editor.plugins.LinkDialog");
dojo.require("dijit._editor.plugins.FontChoice");
dojo.require("dijit.ProgressBar");

dojo.require("dijit.form.ComboBox");
dojo.require("dijit.form.CheckBox");
dojo.require("dijit.form.Textarea");
dojo.require("dijit.form.TextBox");
dojo.require("dijit.form.ComboBox");
dojo.require("dijit.form.FilteringSelect");
dojo.require("dijit.form.Textarea");

dojo.require("dijit.layout.BorderContainer");
dojo.require("dijit.layout.AccordionContainer");
dojo.require("dijit.layout.TabContainer");
dojo.require("dijit.layout.ContentPane");

dojo.require("dojox.grid.DataGrid");
dojo.require("dojox.widget.FisheyeLite");

dojo.addOnLoad(function(){

	dojo.parser.parse();
	dijit.setWaiRole(dojo.body(), "application");

	var n = dojo.byId("preLoader");
	dojo.fadeOut({
		node:n,
		duration:720,
		onEnd:function(){
			// dojo._destroyElement(n); 
			dojo.style(n,"display","none");
		}
	}).play();

	// Set up handler so that when contact information is updated the "display field"
	// (used by "To" field drop-down") is also updated
	dojo.connect(contactStore, "onSet", function(/* item */ item, 
					/* attribute-name-string */ attribute, 
					/* object | array */ oldValue,
					/* object | array */ newValue){
		if(attribute != "display"){
			contactStore.setValue(item, "display", 
					contactStore.getValue(item, "first") + " " +
					contactStore.getValue(item, "last") + " <" +
					contactStore.getValue(item, "email") + ">");
		}		
	});
	
	// make tooltips go down (from buttons on toolbar) rather than to the right
	dijit.Tooltip.defaultPosition = ["above", "below"];
	
	// Write A-Z "links" on contactIndex tab to do filtering
	genIndex();

});

function genIndex(){
	// summary:  generate A-Z push buttons for navigating contact list
	var ci = dojo.byId("contactIndex");
	
	function addChar(c, func, cls){
		// add specified character, when clicked will execute func
		var span = document.createElement("span");
		span.innerHTML = c;
		span.className = cls || "contactIndex";
		ci.appendChild(span);
		new dojox.widget.FisheyeLite(
			{
				properties: {fontSize: 1.5}, 	
				easeIn: dojo.fx.easing.linear,
				durationIn: 100,
				easeOut: dojo.fx.easing.linear,
				durationOut: 100
			},
			span
		);
		dojo.connect(span, "onclick", func || function(){ contactTable.setQuery({first: c+"*"}, {ignoreCase: true}) });
		dojo.connect(span, "onclick", function(){
			dojo.query(">", ci).removeClass("contactIndexSelected");
			dojo.addClass(span, "contactIndexSelected");
		});
	}

	addChar("ALL", function(){contactTable.setQuery({})}, 'contactIndexAll' );
	for(var l = "A".charCodeAt(0); l <= "Z".charCodeAt(0); l++){
		addChar(String.fromCharCode(l))
	}
	addChar("ALL", function(){contactTable.setQuery({})}, 'contactIndexAll' );
}

var paneId = 1;

function onMessageClick(cell){
	// summary: when user clicks a row in the message list pane
	var item = cell.grid.getItem(cell.rowIndex),
		sender = this.store.getValue(item, "sender"),
		subject = this.store.getValue(item, "label"),
		sent = dojo.date.locale.format(
				dojo.date.stamp.fromISOString(this.store.getValue(item, "sent")),
				{formatLength: "long", selector: "date"}),
		text = this.store.getValue(item, "text"),
		messageInner = "<span class='messageHeader'>From: " + sender + "<br>" +
		"Subject: "+ subject + "<br>" +
		"Date: " + sent + "<br><br></span>" +
		text;
	dijit.byId("message").setContent(messageInner);	
}

function searchMessages(){
	// summary: do a custom search for messages across inbox folders
	var query = {type: "message"};
	var searchCriteria = searchForm.attr('value');
	for(var key in searchCriteria){
		var val = searchCriteria[key];
		if(val){
			query[key] = "*" + val + "*";
		}
		table.setQuery(query, {ignoreCase: true});
	}
}

// for "new message" tab closing
function testClose(pane,tab){
  return confirm("Are you sure you want to leave your changes?");
}

// fake mail download code:
var numMails;
var updateFetchStatus = function(x){
	if(x == 0){
		dijit.byId('fakeFetch').update({ indeterminate: false });
		return;
	}
	dijit.byId('fakeFetch').update({ progress: x + 1 });
	if(x == numMails){
		dojo.fadeOut({ node: 'fetchMail', duration:800,
			// set progress back to indeterminate. we're cheating, because this
			// doesn't actually have any data to "progress"
			onEnd: function(){ 
				dijit.byId('fakeFetch').update({ indeterminate: true });
				dojo.byId('fetchMail').style.visibility='hidden'; // remove progress bar from tab order
			}
		}).play();
	}
}
var fakeReport = function(percent){
	// FIXME: can't set a label on an indeterminate progress bar
	// like if(this.indeterminate) { return " connecting."; }
	return dojo.string.substitute("Fetching: ${0} of ${1} messages.", [percent * this.maximum, this.maximum]);
}

var fakeDownload = function(){
	dojo.byId('fetchMail').style.visibility='visible';
	numMails = Math.floor(Math.random()*10) + 1;
	dijit.byId('fakeFetch').update({ maximum: numMails, progress:0 });
	dojo.fadeIn({ node: 'fetchMail', duration:300 }).play();
	for(var ii = 0; ii < numMails + 1; ++ii){
		var func = dojo.partial(updateFetchStatus, ii);
		setTimeout(func,  ((ii + 1) * (Math.floor(Math.random()*100) + 400)));
	}
}

// fake sending dialog progress bar 
var stopSendBar = function(){
	dijit.byId('fakeSend').update({ indeterminate: false });
	dijit.byId('sendDialog').hide();
	tabs.selectedChildWidget.onClose = function(){return true;};  // don't want confirm message 
	tabs.closeChild(tabs.selectedChildWidget);
}
	 
var showSendBar = function(){
	dijit.byId('fakeSend').update({ indeterminate: true });
	dijit.byId('sendDialog').show();
	setTimeout(function(){stopSendBar();}, 3000);
}

