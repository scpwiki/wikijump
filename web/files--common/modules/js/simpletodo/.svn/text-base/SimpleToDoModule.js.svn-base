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

WIKIDOT.modules.SimpleToDoModule= {} ;

WIKIDOT.modules.SimpleToDoModule.callbacks = {
	save: function(data) {
		var serializedData = JSON.stringify(data);
		var p = {data:serializedData} ;
		p.pageId = WIKIREQUEST.info.pageId;
		p.action = "SimpleToDoAction";
		p.event = "save";
		OZONE.ajax.requestModule(null,p,WIKIDOT.modules.SimpleToDoModule.callbacks.saveCallback);
	},
	
	saveCallback: function(r){
		if(!WIKIDOT.utils.handleError(r)) {return;}
	}
}

WIKIDOT.modules.SimpleToDoModule.listeners = {
	
	clickCheckBoxToChangeState: function(e){
		if (WIKIDOT.modules.SimpleToDoModule.utils.editPermission.innerHTML == "true"){
			var checkBoxState ;
			var myDocument = document ;
			var checkBoxElement ;
			if (!e) var e = window.event
			if (e.target) checkBoxElement = e.target
			else if (e.srcElement) checkBoxElement = e.srcElement
			if (checkBoxElement.nodeType == 3)
			checkBoxElement = checkBoxElement.parentNode;
			
			if(checkBoxElement.checked == true)	{
				checkBoxState = true ;
			} else {
				checkBoxState = false ;
			}
			var dataToSave = WIKIDOT.modules.SimpleToDoModule.utils.getDataToSerialize(checkBoxElement,myDocument);
			WIKIDOT.modules.SimpleToDoModule.callbacks.save(dataToSave);
		} else {
			alert('You do not have permissions to edit list.');
		}
	},
			
	clickTitleToEdit: function(e){
		if (WIKIDOT.modules.SimpleToDoModule.utils.editPermission.innerHTML == "true"){
			var myDocument = document ;
			var titleElement ;
			if (!e) var e = window.event
			if (e.target) titleElement = e.target
			else if (e.srcElement) titleElement = e.srcElement
			if (titleElement.nodeType == 3)
			titleElement = titleElement.parentNode;		
			YAHOO.util.Event.removeListener(titleElement,"click",WIKIDOT.modules.SimpleToDoModule.listeners.clickTitleToEdit)	;					
			var newTitleTextField = myDocument.createElement("input") ;
			newTitleTextField.type = "text";
			newTitleTextField.value = titleElement.firstChild.nodeValue;
			newTitleTextField.oldValue = titleElement.firstChild.nodeValue;
			YAHOO.util.Event.addListener(newTitleTextField,"blur",WIKIDOT.modules.SimpleToDoModule.listeners.onblurSaveTitle);
			var oldTitleContentNode = titleElement.firstChild;
			YAHOO.util.Event.addListener(newTitleTextField,"keypress",WIKIDOT.modules.SimpleToDoModule.listeners.onKeyPressSaveTitle); 			
			YAHOO.util.Dom.addClass(newTitleTextField,"text");
			titleElement.removeChild(oldTitleContentNode);
			titleElement.appendChild(newTitleTextField);
			newTitleTextField.style.width = "90%";
			newTitleTextField.style.textAlign = "center";
			newTitleTextField.focus();
		}
	},

	onblurSaveTitle: function(e) {
		if (WIKIDOT.modules.SimpleToDoModule.utils.editPermission.innerHTML == "true"){		
			var myDocument = document ;
			var titleElement ;
			if (!e) var e = window.event
			if (e.target) titleElement = e.target
			else if (e.srcElement) titleElement = e.srcElement
			if (titleElement.nodeType == 3)
			titleElement = titleElement.parentNode;
			YAHOO.util.Event.removeListener(titleElement.parentNode,"blur",WIKIDOT.modules.SimpleToDoModule.listeners.onblurSaveTitle);
			YAHOO.util.Event.addListener(titleElement.parentNode,"click",WIKIDOT.modules.SimpleToDoModule.listeners.clickTitleToEdit);
			if (titleElement.nodeName == "INPUT"){
				titleElement.value = titleElement.value.replace(/^\s+/,'');
				titleElement.value = titleElement.value.replace(/\s+$/,'');
				var titleDefaultText = myDocument.getElementById("simpletodo-data-title");
				var pTitleElement=titleElement.parentNode ;
				if (titleElement.oldValue != titleElement.value){
					
					if (titleElement.value != ""){
						
						var newTextNode = myDocument.createTextNode(titleElement.value); 
						
					} else {
						
						var newTextNode = myDocument.createTextNode(titleDefaultText.innerHTML);
						
					}
					pTitleElement.removeChild(titleElement);
					pTitleElement.appendChild(newTextNode);
					//AJAX REQUEST
					var dataToSave = WIKIDOT.modules.SimpleToDoModule.utils.getDataToSerialize(pTitleElement,myDocument);
					WIKIDOT.modules.SimpleToDoModule.callbacks.save(dataToSave);
					
				} else {
					
					if (titleElement.value != ""){
						
						var newTextNode = myDocument.createTextNode(titleElement.value); 
						
					} else {
						
						var newTextNode = myDocument.createTextNode(titleDefaultText.innerHTML);
						
					}
					pTitleElement.removeChild(titleElement);
					pTitleElement.appendChild(newTextNode);					
					
				}
			}
		}
	},
	
	clickToEditTask: function(e){
		if (WIKIDOT.modules.SimpleToDoModule.utils.editPermission.innerHTML == "true"){
			var myDocument = document ;
			if (!e) var e = window.event
			if (e.target) var taskElement = e.target
			else if (e.srcElement) var taskElement = e.srcElement
	
			if (taskElement.nodeName == "SPAN"){
				YAHOO.util.DragDropMgr.lock();
				var parentElement = taskElement.parentNode ;
				var newToDoTextField = myDocument.createElement("input");
				newToDoTextField.type = "text";
				YAHOO.util.Event.addListener(newToDoTextField, 'blur',WIKIDOT.modules.SimpleToDoModule.listeners.onblurSaveTask);
				newToDoTextField.value = taskElement.firstChild.nodeValue;
				newToDoTextField.oldValue = taskElement.firstChild.nodeValue; 
			    YAHOO.util.Dom.addClass(taskElement.parentNode.parentNode.parentNode,"edit");
				var pTaskElement = taskElement.firstChild ; 			
				var width = pTaskElement.parentNode.offsetWidth ; 
				parentElement.removeChild(taskElement);
				parentElement.appendChild(newToDoTextField);
				YAHOO.util.Dom.addClass(newToDoTextField,"text");
				YAHOO.util.Event.addListener(newToDoTextField,"keypress",WIKIDOT.modules.SimpleToDoModule.listeners.onKeyPressSaveTask);
				newToDoTextField.style.width = (Math.max(width,100))+"px" ;
				newToDoTextField.focus() ;
				//dirty hack firefox bug workaround
				var tmp_id = YAHOO.util.Dom.generateId(newToDoTextField,"tmp_hack");
				setTimeout("$('"+tmp_id+"').focus()",50);
			}
		}
	},
	
	onblurSaveTask: function(e){
		if (WIKIDOT.modules.SimpleToDoModule.utils.editPermission.innerHTML == "true"){
			var myDocument = document ;
			var taskElement ;
			if (!e) var e = window.event
			if (e.target) taskElement = e.target
			else if (e.srcElement) taskElement = e.srcElement
			if (taskElement.nodeType == 3)
			taskElement = taskElement.parentNode;		
			var parentElement = taskElement.parentNode ;
			var optionsElement  = parentElement.getElementsByTagName("span");
			YAHOO.util.Dom.removeClass(taskElement.parentNode.parentNode.parentNode,"edit");
			if (taskElement.nodeName == "INPUT"){
				taskElement.value = taskElement.value.replace(/^\s+/,'');
				taskElement.value = taskElement.value.replace(/\s+$/,'');
				var taskDefaultText = myDocument.getElementById("simpletodo-data-itemtext");
				var pTaskElement=taskElement.parentNode ;
				if (taskElement.oldValue != taskElement.value){
					if (taskElement.value != ""){
						var newTextNode = myDocument.createTextNode(taskElement.value); 
					} else {
						var newTextNode = myDocument.createTextNode(taskDefaultText.innerHTML) ;
					}
					var newSpanNode = myDocument.createElement("span");
					newSpanNode.appendChild(newTextNode);
					YAHOO.util.Dom.addClass(newSpanNode,"text");
					pTaskElement.removeChild(taskElement);
					pTaskElement.appendChild(newSpanNode);
					//AJAX REQUEST
					var dataToSave = WIKIDOT.modules.SimpleToDoModule.utils.getDataToSerialize(pTaskElement,myDocument);
					WIKIDOT.modules.SimpleToDoModule.callbacks.save(dataToSave);
				} else {
					
					if (taskElement.value != ""){
						var newTextNode = myDocument.createTextNode(taskElement.value); 
					} else {
						var newTextNode = myDocument.createTextNode(taskDefaultText.innerHTML) ;
					}
					var newSpanNode = myDocument.createElement("span");
					newSpanNode.appendChild(newTextNode);
					YAHOO.util.Dom.addClass(newSpanNode,"text");
					pTaskElement.removeChild(taskElement);
					pTaskElement.appendChild(newSpanNode);
					
				}
			}
			
			YAHOO.util.DragDropMgr.unlock();
		}
	},
	
	clickToRemoveTask: function(e){
		if (WIKIDOT.modules.SimpleToDoModule.utils.editPermission.innerHTML == "true"){
			var myDocument = document ;
			var taskElement ;
			if (!e) var e = window.event
			if (e.target) taskElement = e.target
			else if (e.srcElement) taskElement = e.srcElement
			if (taskElement.nodeType == 3)
			taskElement = taskElement.parentNode; 
			var itemToRemove = WIKIDOT.modules.SimpleToDoModule.utils.getDirectParentNodeByName("div",taskElement) ;
			var parentElement =itemToRemove.parentNode ;
			parentElement.removeChild(itemToRemove);
			//AJAX REQUEST
			var dataToSave = WIKIDOT.modules.SimpleToDoModule.utils.getDataToSerialize(parentElement,myDocument);
			WIKIDOT.modules.SimpleToDoModule.callbacks.save(dataToSave);
		}
	},
	
	clickToAddTask : function(e,parentId){
		if (WIKIDOT.modules.SimpleToDoModule.utils.editPermission.innerHTML == "true"){
			var myDocument = document ;
			var taskElement ;
			if (!e) var e = window.event
			if (e.target) taskElement = e.target
			else if (e.srcElement) taskElement = e.srcElement
			if (taskElement.nodeType == 3) 
			taskElement = taskElement.parentNode;	
			var itemToAdd = WIKIDOT.modules.SimpleToDoModule.utils.createToDoItem(myDocument);
			YAHOO.util.Dom.generateId(itemToAdd,"simpletodo-task");
			var topElement = myDocument.getElementById(parentId.toString());
			var parentElement = YAHOO.util.Dom.getElementsByClassName("simpletodo-sub-box","div",topElement);
			parentElement[0].appendChild(itemToAdd);
			new WIKIDOT.modules.SimpleToDoModule.DD.DDList(itemToAdd.id.toString());
		}
	},
	
	onKeyPressSaveTask: function(e){
		if (WIKIDOT.modules.SimpleToDoModule.utils.editPermission.innerHTML == "true"){
			if (e.keyCode == 13){
				var myDocument = document ;
				var taskElement ;
				if (!e) var e = window.event
				if (e.target) taskElement = e.target
				else if (e.srcElement) taskElement = e.srcElement
				if (taskElement.nodeType == 3)
				taskElement = taskElement.parentNode;		
				var parentElement = taskElement.parentNode ;
				var optionsElement  = parentElement.getElementsByTagName("span");
				YAHOO.util.Dom.removeClass(taskElement.parentNode.parentNode.parentNode,"edit");
				if (taskElement.nodeName == "INPUT"){
					taskElement.value = taskElement.value.replace(/^\s+/,'');
					taskElement.value = taskElement.value.replace(/\s+$/,'');
					var taskDefaultText = myDocument.getElementById("simpletodo-data-itemtext");
					var pTaskElement=taskElement.parentNode ;
					if (taskElement.oldValue != taskElement.value){
						if (taskElement.value != ""){
							var newTextNode = myDocument.createTextNode(taskElement.value); 
						} else {
							var newTextNode = myDocument.createTextNode(taskDefaultText.innerHTML) ;
						}
						var newSpanNode = myDocument.createElement("span");
						newSpanNode.appendChild(newTextNode);
						YAHOO.util.Dom.addClass(newSpanNode,"text");
						pTaskElement.removeChild(taskElement);
						pTaskElement.appendChild(newSpanNode);
						//AJAX REQUEST
						var dataToSave = WIKIDOT.modules.SimpleToDoModule.utils.getDataToSerialize(pTaskElement,myDocument);
						WIKIDOT.modules.SimpleToDoModule.callbacks.save(dataToSave);
					} else {
						
						if (taskElement.value != ""){
							var newTextNode = myDocument.createTextNode(taskElement.value); 
						} else {
							var newTextNode = myDocument.createTextNode(taskDefaultText.innerHTML) ;
						}
						var newSpanNode = myDocument.createElement("span");
						newSpanNode.appendChild(newTextNode);
						YAHOO.util.Dom.addClass(newSpanNode,"text");
						pTaskElement.removeChild(taskElement);
						pTaskElement.appendChild(newSpanNode);
						
					}
				}
				
				YAHOO.util.DragDropMgr.unlock();
			}
		}
	},
	
	onKeyPressSaveTitle: function(e){
		if (WIKIDOT.modules.SimpleToDoModule.utils.editPermission.innerHTML == "true"){
			if (e.keyCode == 13){
				var myDocument = document ;
				var titleElement ;
				if (!e) var e = window.event
				if (e.target) titleElement = e.target
				else if (e.srcElement) titleElement = e.srcElement
				if (titleElement.nodeType == 3)
				titleElement = titleElement.parentNode;
				YAHOO.util.Event.removeListener(titleElement.parentNode,"blur",WIKIDOT.modules.SimpleToDoModule.listeners.onblurSaveTitle);
				YAHOO.util.Event.addListener(titleElement.parentNode,"click",WIKIDOT.modules.SimpleToDoModule.listeners.clickTitleToEdit);
				if (titleElement.nodeName == "INPUT"){
					titleElement.value = titleElement.value.replace(/^\s+/,'');
					titleElement.value = titleElement.value.replace(/\s+$/,'');
					var titleDefaultText = myDocument.getElementById("simpletodo-data-title");
					var pTitleElement=titleElement.parentNode ;
					if (titleElement.oldValue != titleElement.value){
						
						if (titleElement.value != ""){
							
							var newTextNode = myDocument.createTextNode(titleElement.value); 
							
						} else {
							
							var newTextNode = myDocument.createTextNode(titleDefaultText.innerHTML);
							
						}
						pTitleElement.removeChild(titleElement);
						pTitleElement.appendChild(newTextNode);
						//AJAX REQUEST
						var dataToSave = WIKIDOT.modules.SimpleToDoModule.utils.getDataToSerialize(pTitleElement,myDocument);
						WIKIDOT.modules.SimpleToDoModule.callbacks.save(dataToSave);
						
					} else {
						
						if (titleElement.value != ""){
							
							var newTextNode = myDocument.createTextNode(titleElement.value); 
							
						} else {
							
							var newTextNode = myDocument.createTextNode(titleDefaultText.innerHTML);
							
						}
						pTitleElement.removeChild(titleElement);
						pTitleElement.appendChild(newTextNode);					
						
					}
				}	
			}
		}
	},
	
	clickToEditLink: function (e){
		if (WIKIDOT.modules.SimpleToDoModule.utils.editPermission.innerHTML == "true"){
			var myDocument = document ;
			var linkEl ;
			if (!e) var e = window.event
			if (e.target) linkEl = e.target
			else if (e.srcElement) linkEl = e.srcElement
			if (linkEl.nodeType == 3)
			linkEl = linkEl.parentNode; //to jest span
			//tu bede sprawdzal czy link byl wczesniej ustawiony na jakas konkretna wartosc a jezeli tak
			//to przy edycji bedzie ta wartosc  pojawiac sie w inpucie
			var parentElement = linkEl.parentNode.parentNode ; //to ma byc task i jest :-)
			var linkElement = WIKIDOT.modules.SimpleToDoModule.utils.createLinkField(myDocument) ; //tu juz dodawalem event :-)		
			var linkToFollow = parentElement.getElementsByTagName("a");
			var linkElementSpans = linkElement.getElementsByTagName("span");
			if (linkToFollow[0].href.match(/^http/)){	//bo teaz sa 4 elemeny a
						linkElementSpans[1].firstChild.value = linkToFollow[0].href ;
			}
			parentElement.appendChild(linkElement);
			linkElementSpans = linkElement.getElementsByTagName("span");
			var linkInput = linkElementSpans[1].getElementsByTagName("input");
			linkInput.item(0).focus();
			YAHOO.util.DragDropMgr.lock();
			/*
			 * 
			 * Teraz troche bajzlu // to jest ten AUTOCOMPLEATER
			 */
			var myDataSource = new YAHOO.widget.DS_XHR("/quickmodule.php", ['pages', 'unix_name', 'title']); 
			myDataSource.scriptQueryParam="q";
			myDataSource.scriptQueryAppend = "s="+WIKIREQUEST.info.siteId+"&module=PageLookupQModule";
		
			var myAutoComp = new YAHOO.widget.AutoComplete("link-page-name","link-page-name-list", myDataSource);
			myAutoComp.formatResult = function(aResultItem, sQuery) { 
			var title = aResultItem[1];
			var unixName = aResultItem[0];
			if(unixName!= null){
			return '<div style="font-size: 100%">'+unixName+'</div><div style="font-size: 85%;">('+title+')</div>';
			} else {
			return "";
			}
			}
			myAutoComp.minQueryLength = 2;
			myAutoComp.queryDelay = 0.5;
	
			/*
			 * 
			 * KOniec bajzlu
			 * 
			 * 
			 */
	
		}
	},
	
	onblurSaveLink: function (e){
		if (WIKIDOT.modules.SimpleToDoModule.utils.editPermission.innerHTML == "true"){
			var myDocument = document ;
			var linkEl ;
			if (!e) var e = window.event
			if (e.target) linkEl = e.target
			else if (e.srcElement) linkEl = e.srcElement
			if (linkEl.nodeType == 3)
			linkEl = linkEl.parentNode;
			var linkBox = WIKIDOT.modules.SimpleToDoModule.utils.getDirectParentNodeByName("div",linkEl) ;
			var taskBox = WIKIDOT.modules.SimpleToDoModule.utils.getDirectParentNodeByName("div",linkBox) ;
			var linkBoxSpans = linkBox.getElementsByTagName("span")	;
			var taskBoxSpans = YAHOO.util.Dom.getElementsByClassName("follow-link","span",taskBox);
			var taskBoxAnchors = taskBoxSpans[0].getElementsByTagName("a");
			linkBoxSpans[1].firstChild.value = linkBoxSpans[1].firstChild.value.replace(/^\s+/,''); //input value
			linkBoxSpans[1].firstChild.value = linkBoxSpans[1].firstChild.value.replace(/\s+$/,''); //input value
			var userLinkValue = linkBoxSpans[1].firstChild.value.toString() ; //zbieram z pola tekstowego wartosc
			if ((userLinkValue != "javascript:;") && (userLinkValue != '')){
				YAHOO.util.Dom.addClass(taskBoxAnchors[0].parentNode,"proper-link");
				taskBoxAnchors[0].href = userLinkValue ;
			}else{
				YAHOO.util.Dom.removeClass(taskBoxAnchors[0].parentNode,"proper-link");
				taskBoxAnchors[0].href = 'javascript:;' ;
			}
			taskBox.removeChild(linkBox);
			YAHOO.util.DragDropMgr.unlock();
			//AJAX REQUEST
			var dataToSave = WIKIDOT.modules.SimpleToDoModule.utils.getDataToSerialize(taskBox,myDocument);
			WIKIDOT.modules.SimpleToDoModule.callbacks.save(dataToSave);
		}
	},
	
	onKeyPressSaveLink: function (e){
		if (WIKIDOT.modules.SimpleToDoModule.utils.editPermission.innerHTML == "true"){
			if (e.keyCode == 13){
				var myDocument = document ;
				var linkEl ;
				if (!e) var e = window.event
				if (e.target) linkEl = e.target
				else if (e.srcElement) linkEl = e.srcElement
				if (linkEl.nodeType == 3)
				linkEl = linkEl.parentNode;
				var linkBox = WIKIDOT.modules.SimpleToDoModule.utils.getDirectParentNodeByName("div",linkEl) ;
				var taskBox = WIKIDOT.modules.SimpleToDoModule.utils.getDirectParentNodeByName("div",linkBox) ;
				var linkBoxSpans = linkBox.getElementsByTagName("span")	;
				var taskBoxSpans = YAHOO.util.Dom.getElementsByClassName("follow-link","span",taskBox);
				var taskBoxAnchors = taskBoxSpans[0].getElementsByTagName("a");
				linkBoxSpans[1].firstChild.value = linkBoxSpans[1].firstChild.value.replace(/^\s+/,''); //input value
				linkBoxSpans[1].firstChild.value = linkBoxSpans[1].firstChild.value.replace(/\s+$/,''); //input value
				var userLinkValue = linkBoxSpans[1].firstChild.value.toString() ; //zbieram z pola tekstowego wartosc
				if ((userLinkValue != "javascript:;") && (userLinkValue != '')){
				YAHOO.util.Dom.addClass(taskBoxAnchors[0].parentNode,"proper-link");
				taskBoxAnchors[0].href = userLinkValue ;
				}else{
					YAHOO.util.Dom.removeClass(taskBoxAnchors[0].parentNode,"proper-link");
					taskBoxAnchors[0].href = 'javascript:;' ;
				}
				taskBox.removeChild(linkBox);
				YAHOO.util.DragDropMgr.unlock();
				//AJAX REQUEST
				var dataToSave = WIKIDOT.modules.SimpleToDoModule.utils.getDataToSerialize(taskBox,myDocument);
				WIKIDOT.modules.SimpleToDoModule.callbacks.save(dataToSave);
			}

		}
	}
};

/*
 * Inicjalizacja poszczegolnych itemow z listy Pierwsze wywolanie
 * 
 */

/*
 * 
 * 
 * 
 * DRAG AND DROP IMPLEMENTATION USING YAHOO UI
 * SOME CHANGES ARE NEEDED
 * 
 * 
 * 
 * 
 */
 WIKIDOT.modules.SimpleToDoModule.DD = {};

OZONE.dom.onDomReady(function(){

	var myConfigs = {
    width: "300px", // Width of console
    height: "200", // Height of container
    left: "2%", // Position from left edge of viewport
    top: "60%", // Position from top edge of viewport
    right: "30em", // Position from right edge of viewport
    bottom: "40%", // Position from bottom edge of viewport
    fontSize: "120%", // Increase default font size
    footerEnabled: true, // Don't show filters/pause/resume/clear UI
    logReaderEnabled: true, // Pause right away
    thresholdMax: 100, // Show a maximum of 100 messages in the console
    thresholdMin: 10, // When thresholdMax is reached, clear out all messages
                      // in the console except the last 10
    draggable: true, // If DragDrop utility is present, LogReader can be dragged
    outputBuffer: 100 // Logs get written to LogReader with a 100 millisecond buffer

   	};
	var myContainer = null; // LogReader will create markup from scratch
			
/**
 * 
 * INIT INIT INIT INIT INIT INIT
 * 
 * 
 */
(function(){  //Init 
	WIKIDOT.modules.SimpleToDoModule.utils.editPermission = document.getElementById("simpletodo-data-edit-permission");
	var arrayOfLists = YAHOO.util.Dom.getElementsByClassName("simpletodo-box");
	var arrayOfItems ;	
	var t ;
	for (var i=0; i < arrayOfLists.length; i++){
		t = YAHOO.util.Dom.getElementsByClassName("title",null,arrayOfLists[i]);
		YAHOO.util.Event.addListener(t[0],"click",WIKIDOT.modules.SimpleToDoModule.listeners.clickTitleToEdit);
		arrayOfItems = YAHOO.util.Dom.getElementsByClassName("task","div",arrayOfLists[i]) ;
		YAHOO.util.Dom.generateId(arrayOfItems,"simpletodo-task");
		for (var j=0; j < arrayOfItems.length; j++){
			var checkBox = YAHOO.util.Dom.getElementsByClassName("checkbox","input",arrayOfItems[j]) ;	
			var text = arrayOfItems[j].getElementsByTagName("span");
			var anchor = arrayOfItems[j].getElementsByTagName("a");
			YAHOO.util.Event.addListener(checkBox[0],"click",WIKIDOT.modules.SimpleToDoModule.listeners.clickCheckBoxToChangeState);
			YAHOO.util.Event.addListener(text[1],"click",WIKIDOT.modules.SimpleToDoModule.listeners.clickToEditTask);
			YAHOO.util.Event.addListener(anchor[2],"click",WIKIDOT.modules.SimpleToDoModule.listeners.clickToRemoveTask);
			YAHOO.util.Event.addListener(anchor[1],"click",WIKIDOT.modules.SimpleToDoModule.listeners.clickToEditLink);
			
		}
		
	}
	WIKIDOT.modules.SimpleToDoModule.utils.ieHoverFix("simpletodo-sub-box");
	if (WIKIDOT.modules.SimpleToDoModule.utils.editPermission.innerHTML == "false") {
		YAHOO.util.DragDropMgr.lock()
	}
}
)();

/*
 * Here is a place for drag and drop
 */

(function() {

var Dom = YAHOO.util.Dom;
var Event = YAHOO.util.Event;
var DDM = YAHOO.util.DragDropMgr;
//////////////////////////////////////////////////////////////////////////////
// example app
//////////////////////////////////////////////////////////////////////////////
WIKIDOT.modules.SimpleToDoModule.DD.DDApp = {
    init: function() {
		var arrayOfLists = YAHOO.util.Dom.getElementsByClassName("simpletodo-sub-box");
        var i,j;
        
        for (i=0;i<arrayOfLists.length;i=i+1) {
            new YAHOO.util.DDTarget(""+arrayOfLists[i].id);
       		var items = YAHOO.util.Dom.getElementsByClassName("task","div",arrayOfLists[i]);
            for (j=0; j < items.length; j++){
            	new WIKIDOT.modules.SimpleToDoModule.DD.DDList(""+items[j].id);
            }
            
        }      
    }

};

//////////////////////////////////////////////////////////////////////////////
// custom drag and drop implementation
//////////////////////////////////////////////////////////////////////////////

WIKIDOT.modules.SimpleToDoModule.DD.DDList = function(id, sGroup, config) {
	
    WIKIDOT.modules.SimpleToDoModule.DD.DDList.superclass.constructor.call(this, id, sGroup, config);
    var el = this.getDragEl();
    Dom.setStyle(el, "opacity", 0.67); // The proxy is slightly transparent
    this.goingUp = false;
    this.lastY = 0;
    
};

YAHOO.extend(WIKIDOT.modules.SimpleToDoModule.DD.DDList, YAHOO.util.DDProxy, {

    startDrag: function(x, y) {
        var dragEl = this.getDragEl();
        var clickEl = this.getEl();
        WIKIDOT.modules.SimpleToDoModule.utils.oldDragElement = clickEl ;
   		WIKIDOT.modules.SimpleToDoModule.utils.oldDragElementList = WIKIDOT.modules.SimpleToDoModule.utils.getParentElementOfSpecifiedClass("task","div",clickEl,document);
        var option = Dom.getElementsByClassName("options","span",clickEl);
        var followLink = Dom.getElementsByClassName("follow-link","span",clickEl);
        Dom.setStyle(clickEl, "visibility", "hidden");   //tu jest zaslaniany element z listy ktory zostaje tak na prawde w miejscu
		Dom.setStyle(option[0],"visibility","hidden");
		Dom.setStyle(followLink[0],"visibility","hidden");
        dragEl.innerHTML = clickEl.innerHTML;
        Dom.setStyle(dragEl, "color", Dom.getStyle(clickEl, "color"));
        Dom.setStyle(dragEl, "backgroundColor", Dom.getStyle(clickEl, "backgroundColor"));
        Dom.setStyle(dragEl, "border", "2px solid gray");
    },

    endDrag: function(e) {

        var srcEl = this.getEl();
        var proxy = this.getDragEl();
        // Show the proxy element and animate it to the src element's location
        Dom.setStyle(proxy, "visibility", "");
        var a = new YAHOO.util.Motion( 
            proxy, { 
                points: { 
                    to: Dom.getXY(srcEl)
                }
            }, 
            0.2, 
            YAHOO.util.Easing.easeOut 
        )
        var proxyid = proxy.id;
        var thisid = this.id;
        var option = Dom.getElementsByClassName("options","span",thisid);
        var followLink = Dom.getElementsByClassName("follow-link","span",thisid);
        WIKIDOT.modules.SimpleToDoModule.utils.newDragElementList = WIKIDOT.modules.SimpleToDoModule.utils.getParentElementOfSpecifiedClass("task","div",srcEl,document);
		if (WIKIDOT.modules.SimpleToDoModule.utils.newDragElementList.id != WIKIDOT.modules.SimpleToDoModule.utils.oldDragElementList.id ){
			var dataToSave = WIKIDOT.modules.SimpleToDoModule.utils.getDataToSerialize(WIKIDOT.modules.SimpleToDoModule.utils.oldDragElementList,document);
			WIKIDOT.modules.SimpleToDoModule.callbacks.save(dataToSave);
		}
		
        // Hide the proxy and show the source element when finished with the animation
        a.onComplete.subscribe(function() {
        		Dom.setStyle(option[0],"visibility","");
        		Dom.setStyle(followLink[0],"visibility","");
                Dom.setStyle(proxyid, "visibility", "hidden");
                Dom.setStyle(thisid, "visibility", "");
            });
        a.animate();
        var dataToSave = WIKIDOT.modules.SimpleToDoModule.utils.getDataToSerialize(srcEl,document);
		WIKIDOT.modules.SimpleToDoModule.callbacks.save(dataToSave);
    },

    onDragDrop: function(e, id) {

        // If there is one drop interaction, the div was dropped either on the list,
        // or it was dropped on the current location of the source element.
        if (DDM.interactionInfo.drop.length === 1) {
			
            // The position of the cursor at the time of the drop (YAHOO.util.Point)
            var pt = DDM.interactionInfo.point; 

            // The region occupied by the source element at the time of the drop
            var region = DDM.interactionInfo.sourceRegion; 
			
            // Check to see if we are over the source element's location.  We will
            // append to the bottom of the list once we are sure it was a drop in
            // the negative space (the area of the list without any list items)
            if (!region.intersect(pt)) {
                var destEl = Dom.get(id);
                var destDD = DDM.getDDById(id);
                destEl.appendChild(this.getEl());
                destDD.isEmpty = false;
                DDM.refreshCache();
            }

        }
    },

    onDrag: function(e) {

        // Keep track of the direction of the drag for use during onDragOver
        var y = Event.getPageY(e);

        if (y < this.lastY) {
            this.goingUp = true;
        } else if (y > this.lastY) {
            this.goingUp = false;
        }

        this.lastY = y;
    },

    onDragOver: function(e, id) {
    
        var srcEl = this.getEl();
        var destEl = Dom.get(id);
        // We are only concerned with div items, we ignore the dragover
        // notifications for the list.
        
        if (destEl.nodeName.toLowerCase() == "div" && Dom.hasClass(destEl,"task")) {
            var orig_p = srcEl.parentNode;
            var p = destEl.parentNode;

            if (this.goingUp) {
                p.insertBefore(srcEl, destEl); // insert above
            } else {
                p.insertBefore(srcEl, destEl.nextSibling); // insert below
            }

            DDM.refreshCache();
        }
    }
});

Event.onDOMReady(WIKIDOT.modules.SimpleToDoModule.DD.DDApp.init, WIKIDOT.modules.SimpleToDoModule.DD.DDApp, true);
})();

}, "dummy-ondomready-block");

WIKIDOT.modules.SimpleToDoModule.utils = {
	newDragElement: undefined ,
	
	oldDragElement: undefined ,
	
	newDragElementList: undefined ,
	
	oldDragElementList: undefined ,
	
	linkCounter: 0 ,
	
	editPermission: undefined ,
	
	getDirectParentNodeByName: function (parentName, startEl){
		var walker  = startEl.parentNode ;
		while (walker && ( walker.nodeName.toLowerCase() != parentName.toLowerCase())){
			walker = walker.parentNode ;
		}
		return walker ;
	
	},
	
	ieHoverFix: function (rootClass) { //musze dodac cie przy inicjalizacji i przy dodawaniu nowych elementow :-)
		if (navigator.appName.indexOf("Microsoft") != -1 || navigator.appVersion.indexOf("6.") != -1) {
			var r = YAHOO.util.Dom.getElementsByClassName(rootClass,"div"); //TO JEST TERAZ TABLICA
  			if(r == null){return;}
  			
  			for (var j=0; j < r.length; j++){
  				
  				var item = YAHOO.util.Dom.getElementsByClassName("task","div",r[j]);
  				
  				for (var k=0; k<item.length; k++){
  					
  					YAHOO.util.Event.addListener(item[k],"mouseover",function(e){YAHOO.util.Dom.addClass(this,"iehover");});
  					YAHOO.util.Event.addListener(item[k],"mouseout",function(e){YAHOO.util.Dom.removeClass(this,"iehover");});
  				}
  				
  			}
  			
 			return;
		}		
  	},
  	
  	createToDoItem: function (doc) {
		var myDocument = doc ;
		//Creating nesesary elements
		var newItemDivElement =  myDocument.createElement("div");
		var newItemSpanElement1 = myDocument.createElement("span");  //for checkbox
		var newItemSpanElement2 = myDocument.createElement("span");  //for task text
		var newItemSpanElement2_1 = myDocument.createElement("span");
		var newItemSpanElement3 = myDocument.createElement("span");  //for additional menu
		var newItemSpanElement4 = myDocument.createElement("span");  //for fallow link element
		var newItemCheckBoxElement = myDocument.createElement("input"); //chexbox but needs type attribute
		var newItemDefaultText = myDocument.getElementById("simpletodo-data-itemtext");
		var newItemTextNode = myDocument.createTextNode(newItemDefaultText.innerHTML);
		var newItemAnchorElement = myDocument.createElement("a"); //remove item
		var newItemAnchorElement2 = myDocument.createElement("a"); //edit link
		var newItemAnchorElement3 = myDocument.createElement("a"); //fallow link
		//spany dla anchorow
		var newItemAnchorSpanElement = myDocument.createElement("span");
		var newItemAnchorSpanElement2 = myDocument.createElement("span");
		var newItemAnchorSpanElement3 = myDocument.createElement("span");
		YAHOO.util.Dom.addClass(newItemAnchorElement, "icon2");
		YAHOO.util.Dom.addClass(newItemAnchorElement2, "icon3");
		YAHOO.util.Dom.addClass(newItemAnchorElement3, "icon1");
		YAHOO.util.Dom.addClass(newItemDivElement, "task");
		YAHOO.util.Dom.addClass(newItemSpanElement3, "options");
		YAHOO.util.Dom.addClass(newItemCheckBoxElement, "checkbox");
		YAHOO.util.Dom.addClass(newItemSpanElement1, "checkbox");
		YAHOO.util.Dom.addClass(newItemSpanElement2_1, "text");
		YAHOO.util.Dom.addClass(newItemSpanElement4,"follow-link");
		YAHOO.util.Event.addListener(newItemAnchorElement,"click",WIKIDOT.modules.SimpleToDoModule.listeners.clickToRemoveTask);
		//wrzucam spany dla anchorow wewnatrz nich
		newItemAnchorElement.appendChild(newItemAnchorSpanElement);
		newItemAnchorElement2.appendChild(newItemAnchorSpanElement2);
		newItemAnchorElement3.appendChild(newItemAnchorSpanElement3);
		newItemAnchorSpanElement.innerHTML = "Remove";
		newItemAnchorSpanElement2.innerHTML = "Edit Link";
		newItemAnchorSpanElement3.innerHTML = "Follow Link";
		newItemAnchorElement.href = "javascript:;";
		newItemAnchorElement2.href = "javascript:;";
		newItemAnchorElement3.href = "javascript:;";
		newItemCheckBoxElement.setAttribute("type","checkbox");
		YAHOO.util.Event.addListener(newItemSpanElement2,"click",WIKIDOT.modules.SimpleToDoModule.listeners.clickToEditTask);
		YAHOO.util.Event.addListener(newItemAnchorElement2,"click",WIKIDOT.modules.SimpleToDoModule.listeners.clickToEditLink);
     	YAHOO.util.Event.addListener(newItemDivElement,"mouseover",function(e){YAHOO.util.Dom.addClass(this,"iehover");});
  		YAHOO.util.Event.addListener(newItemDivElement,"mouseout",function(e){YAHOO.util.Dom.removeClass(this,"iehover");});
		//Starting to building element from scratch
		newItemSpanElement2_1.appendChild(newItemTextNode);
		newItemSpanElement2.appendChild(newItemSpanElement2_1);
		newItemSpanElement3.appendChild(newItemAnchorElement2);
		newItemSpanElement3.appendChild(newItemAnchorElement);
		newItemSpanElement4.appendChild(newItemAnchorElement3); //tu wstawilem follow linka przed menu z opcjami
		newItemSpanElement1.appendChild(newItemCheckBoxElement);
		newItemDivElement.appendChild(newItemSpanElement1);
		newItemDivElement.appendChild(newItemSpanElement2);
		newItemDivElement.appendChild(newItemSpanElement4); //tu wstawilem follow linka przed menu z opcjami
		newItemDivElement.appendChild(newItemSpanElement3);
		
		return newItemDivElement;

	},
	
	createLinkField: function (doc) {
		var myDocument = doc ;
		var newItemDivElement = myDocument.createElement("div"); //content wrap
		var newAutoCompleteDivElement= myDocument.createElement("div");
		var newSpanElement1 = myDocument.createElement("span"); //for Link text.
		var newSpanElement2 = myDocument.createElement("span"); //for input field wrap
		var newSpanElement3 = myDocument.createElement("span"); //for save button
		var newTextInputElement = myDocument.createElement("input");  //for link text
		newTextInputElement.type = "text" ;
		newTextInputElement.value = "";
		newTextInputElement.name = "linksList";
		newTextInputElement.id = "link-page-name";
		newAutoCompleteDivElement.id = "link-page-name-list";
		YAHOO.util.Dom.addClass(newAutoCompleteDivElement, "autocomplete-list");
		YAHOO.util.Dom.addClass(newItemDivElement, "add-link");
		YAHOO.util.Dom.addClass(newItemDivElement, "autocomplete-container"); //potrzebne do auto uzupelniania
		YAHOO.util.Dom.addClass(newSpanElement1, "text");
		YAHOO.util.Dom.addClass(newSpanElement2, "text");
		YAHOO.util.Dom.addClass(newSpanElement3, "text");
		YAHOO.util.Dom.addClass(newTextInputElement, "autocomplete-input"); //do auto podpowiedzi
		YAHOO.util.Dom.addClass(newTextInputElement,"text");
		YAHOO.util.Event.addListener(newTextInputElement,"blur",WIKIDOT.modules.SimpleToDoModule.listeners.onblurSaveLink);
		YAHOO.util.Event.addListener(newTextInputElement,"keypress",WIKIDOT.modules.SimpleToDoModule.listeners.onKeyPressSaveLink);
		newSpanElement2.appendChild(newTextInputElement);
		newSpanElement2.appendChild(newAutoCompleteDivElement);
		newItemDivElement.appendChild(newSpanElement1);
		newItemDivElement.appendChild(newSpanElement2);
		newItemDivElement.appendChild(newSpanElement3);
		
		return newItemDivElement;
		
	},
	
	getDataToSerialize: function(subElement,doc){
	
		var walker = subElement ;
		var myDocument = doc ;
		var _data = new Array() ;

		while (walker){
			if(walker.nodeName == "DIV" && YAHOO.util.Dom.hasClass(walker,"simpletodo-box")){
				var _title = YAHOO.util.Dom.getElementsByClassName("title","div",walker); //there is only one title :] so title[0]
				var _label = YAHOO.util.Dom.getElementsByClassName("label","div",walker); //there is only one label , so label[0]
				var tasks = YAHOO.util.Dom.getElementsByClassName("task","div",walker);
				for (var i=0; i<tasks.length; i++){
					var spans = tasks[i].getElementsByTagName("span");
					var _text = spans[1].firstChild.innerHTML ;
					var _link = spans[3].firstChild.href ;
					var _checked = WIKIDOT.modules.SimpleToDoModule.utils.getCheckBoxValue(spans[0].firstChild) ; //tu trzeba funkji ktora to sprawdzi :-)
					if(_link != "javascript:;" && _link != ''){
						_data[i] = {text: _text, link: _link, checked:_checked};
					} else {
						_data[i] = {text: _text, checked:_checked};
					}
				}
				return {title: _title[0].innerHTML, data:_data, "label":_label[0].innerHTML} ;
			} else {
				walker = walker.parentNode ;	
			}
		}			
	},
	
	getCheckBoxValue: function(checkBox){
		if (checkBox.nodeName.toLowerCase() == "input"){
			if (checkBox.checked == true){
				return true ;
			} else {
				return false ;
			}
		}
	},
	
	getParentElementOfSpecifiedClass: function(parentClassName,parentName,startEl,doc){
		var walker  = startEl.parentNode ;
		
		while ((walker["class"]!=parentClassName) && ( walker.nodeName.toLowerCase() != parentName.toLowerCase())){
			
			walker = walker.parentNode ;
			
		}
		
		return walker ;
	}
	
} ;
