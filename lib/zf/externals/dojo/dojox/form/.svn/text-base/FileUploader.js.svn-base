dojo.provide("dojox.form.FileUploader");
dojo.experimental("dojox.form.FileUploader");

dojox.form.FileUploader = function(options){
	
	//summary: 
	// 		Handles File Uploading to a server (PHP script included for testing)
	//		Does *NOT* create a button, it transforms a button into an uploader. 
	//		This can be used for toolbar buttons for example.
	//
	//		FileUploader is a wrapper class. If the correct version of Flash 
	//		Player is available, FileInputFlash is used. If degradable is true,
	//		and Flash Player is not installed or is outdated, FileInputOverlay
	//		is used.
	//
	// arguments:
	//
	// degradable: /* ? Boolean */
	//		If true, will check if user has the correct version of the 
	//		Flash Player installed, and if not, will cancel FileInputFlash
	//		and install FileInputOverlay instead.
	//		If false and user does not have the correct version of Flash,
	//		(or if user has Opera) FileInputFlash will install regardless,
	//		hopefully triggering the browser update mechanism.
	this.degradable = false;
	//
	//	uploadUrl: /* String */
	// 		REQUIRED: The Url the file will be uploaded
	this.uploadUrl = "";			
	//
	//	button: /* dijit.form.Button or a domNode */
	// 		REQUIRED: The button that when clicked, launches the Flash Upload dialog
	this.button = null;
	//
	// NOTE: 	See FileInputFlash for Flash-specifc options (which will not affect
	//			FileInputOverlay)
	//
	// stubs:
	//
	// 	onChange: function(dataArray){}		// Fired after a file(s) is selected
	// 	onComplete: function(dataArray){}	// Fired after upload complete
	// 
	//	public method(s):
	//
	//	upload()
	//
	if(dojox.embed.Flash.available > 9 && options.degradable && !dojo.isOpera){
		return new dojox.form.FileInputOverlay({
			button:options.button, 
			uploadOnChange:options.uploadOnChange, 
			uploadUrl:options.uploadUrl,
			selectMultipleFiles:options.selectMultipleFiles,
			id:options.id
		});	
	}else{
		return new dojox.form.FileInputFlash({
			button:options.button, 
			uploadOnChange:options.uploadOnChange, 
			uploadUrl:options.uploadUrl,
			id:options.id,
			selectMultipleFiles:options.selectMultipleFiles,
			// Flash specifc arguments:
			fileMask:options.fileMask,
			isDebug:options.isDebug
		});
	}
}



dojo.require("dojox.embed.Flash");

dojo.declare("dojox.form.FileInputFlash", null, {
	//summary: 
	
	//	uploadUrl: /* String */
	// 		The Url the file will be uploaded
	uploadUrl: "",			
	
	//	button: /* dijit.form.Button or a domNode */
	// REQUIRED: The button that when clicked, launches the Flash Upload dialog
	button:null,			
	
	// uploadOnChange: /* Boolean */
	// 		if true, begins upload immediately
	// 		leave false if you wish to display the text of the selection
	//		and present an "upload" button
	uploadOnChange: false, 	
	
	//	fieldName: /* String */
	//			The form field attribute. This will be needed by the server to get the value.
	//			If using the ReceiveFile.php test, leave this as-is.
	// TODO:fieldName:"uploadedfile",
	
	//selectMultipleFiles: /* Boolean */
	// 		Option to restrict to single-file upload, or allow
	// 		multi-file uploader
	selectMultipleFiles:true,
	
	// fileMask: /* Array[ Array[Description, FileTypes], Array[...]...] */
	// 		(an array, or an array of arrays)
	//		Restrict file selection to certain file types
	// Empty array defaults to "All Files"
	//
	// Usage:
	//	fileMask = ["Images", "*.jpg;*.jpeg;*.gif;*.png"]
	//	or
	//	fileMask = [
	//		["Jpeg File", 	"*.jpg;*.jpeg"],
	//		["GIF File", 	"*.gif"],
	//		["PNG File", 	"*.png"],
	//		["All Images", 	"*.jpg;*.jpeg;*.gif;*.png"],
	//	]
	//	NOTE: MacType is not supported, as it does not work very well.
	//			fileMask will work on a Mac, but differently than 
	//			Windows. The second example above in Windows will mask
	//			All but the selected file type, shown in a drop-down at
	//			the bottom of the system dialog. In Mac, all types in 
	//			all arrays will be shown and non-types masked.
	fileMask:[],
	
	// degradable: /* Boolean *?
	//		If true, will check if user has the correct version of the 
	//		Flash Player installed, and if not, will cancel FileInputFlash
	//		and install FileInputOverlay instead.
	//		If false and user does not have the correct version of Flash,
	//		(or if user has Opera) FileInputFlash will install regardless,
	//		hopefully triggering the browser update mechanism.
	degradable: false,
	
	_swfPath: dojo.moduleUrl("dojox.form", "resources/uploader.swf"),
	
	flashObject:null,
	flashMovie:null,
	flashDiv:null,
	
	constructor: function(options){
		console.log("Flash version detected:", dojox.embed.Flash.available);
		this.fileList = [];
		this._subs = [];
		this._cons = [];
		this.button = options.button;
		this.uploadUrl = options.uploadUrl;
		this.uploadOnChange = options.uploadOnChange;
		
		if(this.uploadUrl.toLowerCase().indexOf("http")<0){
			// Appears to be a relative path. Attempt to 
			//	convert it to absolute, so it will better 
			//target the SWF.
			//
			var loc = window.location.href.split("/");
			loc.pop();
			loc = loc.join("/")+"/";
			
			this.uploadUrl = loc+this.uploadUrl;
			console.log("Relative loc:", loc, " abs loc:", this.uploadUrl);
		}
		
		this.selectMultipleFiles = (options.selectMultipleFiles===undefined)?this.selectMultipleFiles:options.selectMultipleFiles;
		this.fileMask = options.fileMask || this.fileMask;
		
		this.id = options.id || dijit.getUniqueId("flashuploader");
		var args = {
			path:this._swfPath.uri,
			width:1,
			height:1,
			// only pass in simple variables - no deep objects
			vars:{
				uploadUrl:this.uploadUrl, 
				uploadOnSelect:this.uploadOnChange,
				selectMultipleFiles:this.selectMultipleFiles,
				id:this.id,
				isDebug:options.isDebug
			}
		};
		//console.log("VARS:", args.vars)
		this.flashDiv = dojo.doc.createElement("div");
		dojo.body().appendChild(this.flashDiv);
		dojo.style(this.flashDiv, "position", "absolute");
		dojo.style(this.flashDiv, "top", "0");
		dojo.style(this.flashDiv, "left", "0");
		
		this._subs.push(dojo.subscribe(this.id+"/filesSelected", this, "_change"));
		this._subs.push(dojo.subscribe(this.id+"/filesUploaded", this, "_complete"));
		this._subs.push(dojo.subscribe(this.id+"/filesProgress", this, "_progress"));
		this._subs.push(dojo.subscribe(this.id+"/filesError", this, "_error"));
		this.flashObject = new dojox.embed.Flash(args, this.flashDiv);
		this.flashObject.onLoad = dojo.hitch(this, function(mov){
			this.flashMovie = mov;
			this.flashMovie.setFileMask(this.fileMask);
		})
		this._cons.push(dojo.connect(this.button, "onClick", this, "_openDialog"));
	},
	
	
	onChange: function(dataArray){
		// summary
		// stub to connect 
		// Fires when files are selected
		// Event is an array of last files selected
	},
	
	onProgress: function(dataArray){
		// summary
		// stub to connect 
		// Fires as progress returns from SWF
		// Event is an array of all files uploading
	},
	
	onComplete: function(dataArray){
		// summary
		// stub to connect 
		// Fires when all files have uploaded
		// Event is an array of all files
	},
	
	onError: function(evtObject){
		console.warn("FLASH/ERROR "+evtObject.type.toUpperCase()+":", evtObject); 	
	},
	
	upload: function(){
		// summary
		// When called, begins file upload 
		this.flashMovie.doUpload();
	},
	
	_error: function(evt){
		this.onError(evt);
	},
	
	_openDialog: function(evt){
		// opens the system dialog
		this.flashMovie.openDialog();
	},
	
	_change: function(dataArray){
		this.fileList = this.fileList.concat(dataArray);
		this.onChange(dataArray);
		if(this.uploadOnChange){
			this.upload();
		}
	},
	
	_complete: function(dataArray){
		for(var i=0;i<this.fileList.length;i++){
			this.fileList[i].percent = 100;	
		}
		this.onProgress(this.fileList);
		this.fileList = [];
		this.onComplete(dataArray);
	},
	
	_progress: function(dataObject){
		for(var i=0;i<this.fileList.length;i++){
			var f = this.fileList[i];
			if(f.name == dataObject.name){
				f.bytesLoaded = dataObject.bytesLoaded;
				f.bytesTotal = dataObject.bytesTotal;
				f.percent = Math.ceil(f.bytesLoaded/f.bytesTotal*100);
			}else{
				if(!f.percent){
					f.bytesLoaded = 0;
					f.bytesTotal = 0;
					f.percent = 0;					
				}
			}
		}
		this.onProgress(this.fileList);
		
	},
	
	destroyAll: function(){
		// Destroys everything including button
		this.button.destroy();
		this.destroy();
	},
	
	destroy: function(){
		//destroys flash
		if(!this.flashMovie){
			this._cons.push(dojo.connect(this, "onLoad", this, "destroy"));	
			return;
		}
		dojo.forEach(this._subs, function(s){
			dojo.unsubscribe(s);								  
		});
		dojo.forEach(this._cons, function(c){
			dojo.disconnect(c);								  
		});
		this.flashObject.destroy();
		dojo._destroyElement(this.flashDiv);
		
	}
	
});


dojo.require("dojo.io.iframe"); 
dojo.require("dojox.html.styles"); 

dojo.experimental("dojox.form.FileInputOverlay");

dojo.declare("dojox.form.FileInputOverlay", null, {
		//summary: 
		// 		Handles the basic tasks of a fileInput...
		//		Does NOT create a button, it transparently overlays a button passed to it. 
		//		This can be used for toolbar buttons for example.
		// 		Handles the file upload. Use an example PHP script included in resources.
		//
		// NOTE:
		//		This looks like it is duplicating efforts of the other FileInput files,
		//		but its actually seperating the lower-level functionality, and allowing
		//		for custom buttons.
		//
		// LIMITATIONS:
		//		Because of the nature of this "hack" - floating a zero-opacity fileInput
		//		over a "fake" button - this won't work in all circumstances. For instance
		//		you couldn't put a fileInput in a scrolling div. Some complicated CSS can 
		//		mess up the placement - or for that matter, some simple, but not expected
		//		CSS can mess up the placement. Being near the botton of a complex document
		//		can throw off the positioning.
		//
		//	OPERA USERS:
		//		Not much love from Opera on FileInput hacks.
		//
		//	ALSO: 
		//		Only works programmatically. Does not work in markup. Use the other
		//		other FileInput files for markup solutions.
		//
		//	USAGE:
		//		this.fileInput = new dojox.form.FileInputOverlay({button:this.button, uploadUrl:this.uploadUrl, uploadOnChange:true});
		//		dojo.connect(this.fileInput, "onChange", this, "handleChange");
		//		dojo.connect(this.fileInput, "onComplete", this, "onComplete");
	
	
		//	_fileInput: /* node */ 
		//	the fileInput form node (do not set)
		_fileInput:null,
		
		//	_fileInput: /* node */ 
		//	the form node (do not set)
		_formNode:null,
		
		//	uploadUrl: /* String */
		// The Url the file will be uploaded
		uploadUrl: "",			
		
		//	button: /* dijit.form.Button or a domNode */
		// REQUIRED: The button that will get the FileInput overlay
		button:null,			
		
		// uploadOnChange: /* Boolean */
		// if true, begins upload immediately
		// leave false if you wish to display the text of the selection
		//	and present an "upload" button
		uploadOnChange: false, 	
		
		//	fieldName: /* String */
		//	The form field attribute. This will be needed by the server to get the value.
		//	If using the ReceiveFile.php test, leave this as-is.
		fieldName:"uploadedfile",
		
		// id: /* String */
		// The attribute of the form field. Also accesses this object.
		id:"",
		
		selectMultipleFiles:false,
								
		constructor: function(options){
			this.button = options.button;
			
			this.uploadUrl = options.uploadUrl;
			this.uploadOnChange = options.uploadOnChange;
			this.selectMultipleFiles = options.selectMultipleFiles,
			this.id = options.id || dijit.getUniqueId("fileInput");
			this.fileCount = 0;
			this._cons = [];
			this.fileInputs = [];
			if(dojo.isIE==6){
				// if we are create more than one FileInputOverlay,
				// IE6 needs a breather or it locks up
				setTimeout(dojo.hitch(this, "createFileInput"), 1);
			}else{
				this.createFileInput();
			}
			
		},
		
		onChange: function(dataArray){
			// summary
			//	Called after a system dialog selection has been made
			// stub to connect
			if(this.uploadOnChange) { 
				this.upload(); 
			}else if(this.selectMultipleFiles){
				this.createFileInput();	
			}
		},
		
		onProgress: function(dataArray){
			// blank stub
			// This won't fire unless you configure the server to 
			//	return a progress. It will fire onComplete at 100%.
		},
		
		onComplete: function(dataArray){
			//
			//stub to connect
			//
			for(var i=0;i<dataArray.length;i++){
				dataArray[i].percent = 100;	
				dataArray[i].name = dataArray[i].file.split("/")[dataArray[i].file.split("/").length-1];
			}
			this.onProgress(dataArray);
			this._removeFileInput();
			this.createFileInput();
		},
		
		upload: function(){
			// summary
			//	Tell form to upload
			dojo.io.iframe.send({
				url: this.uploadUrl,
				form: this._formNode,
				handleAs: "json",
				handle: dojo.hitch(this,function(data,ioArgs,widgetRef){
					this.onComplete(this.selectMultipleFiles?data:[data]);								 
				})
			});
		},
		
		//
		//	File Input Build
		//		
		
		createFileInput: function(){
			// summary
			// Create the fileInput overlay
			//
			
			if(!this.button.id) { this.button.id = dijit.getUniqueId("btn"); }
			var domNode;
			if(this.button.domNode){
				domNode = dojo.byId(this.button.id).parentNode.parentNode;
				// killing this event on the dijit button - it takes over the FileInput
				domNode.parentNode.onmousedown= function(){}
			}else{
				domNode = this.button.parentNode;
			}
			
			this._buildForm(domNode);
			
			this._buildFileInput(domNode);
			
			this.setPosition();	
			
			this._connectInput();
			
			// in some cases, mainly due to scrollbars, the buttons
			//	are initially misplaced
			setTimeout(dojo.hitch(this, "setPosition"), 500);
		},
		
		
		
		setPosition: function(){
			// summary
			//	Get size and location of the 'fake' node (the button)
			//	Resize, set position, and clip the 'real' button (the fileInput)	
			// 	setPosition will  fire on browser resize. The button may wrap to a different position
			//	and sometimes it just shifts slightly in the html, maybe because of the scrollbar.
			// May also call this externally, if there is a change in button positioning.
			//
			
			var fake = this._getFakeButtonSize();
			
			// could memoize this, but it at 2-5ms, doesn't seem quite worth it.
			var real = dojo.marginBox(this._fileInput);
			// Now we have an extremely large fileInput button and field.
			//	We mask the areas that extend passed the boundaries of the button.
			//	Thanks to quirksmode for this hack.
			var clip = "rect(0px "+real.w+"px "+fake.h+"px "+(real.w-fake.w)+"px)";
			this._fileInput.style.clip = clip;
			
			// absolutely position the fileInput.
			this._fileInput.style.left = (fake.x + fake.w - real.w) + "px";
			this._fileInput.style.top = fake.y + "px";
		},
		_getFakeButtonSize: function(){
			// summary
			//	Get the size and position of the Dijit Button or DOM node.
			//	This isn't easy. An awful lot has been accounted for, but a page full
			//	of cascading styles can be simply impossible to predict.
			// In these cases, it's reccomended that this function be 
			//	overwritten with more precise paramters
			//
			var fakeNode = (this.button.domNode) ? dojo.byId(this.button.id).parentNode : dojo.byId(this.button.id);
			
			// can't memoize this, because we need the location. And the size could possibly change anyway.
			var fake = dojo.coords(fakeNode, true);
			// if block, get the width from the style
			fake.w = (dojo.style(fakeNode, "display")=="block")? dojo.style(fakeNode, "width"): fake.w;
			
			//relative and absolute positioning are totally different
			var p = fakeNode.parentNode.parentNode;
			if(p && dojo.style(p, "position")=="relative"){
				fake.x = dojo.style(p, "left");
				fake.y = dojo.style(p, "top");
			}
			if(p && dojo.style(p, "position")=="absolute"){
				fake.x = 0;
				fake.y = 0;
			}
			
			//Tweaking the size of the fileInput to be just a little bigger
			var s = 3;
			fake.x -=s;
			fake.y -= s;
			fake.w+=s*2;
			fake.h+=s*2;
			
			return fake;
		},
		
		
		
		_buildFileInput: function(domNode){
			// summary
			//	Build the fileInput field
			//
			if(this._fileInput){
				//this._formNode.removeChild(this._fileInput);
				this._disconnectInput();
				dojo.style(this._fileInput, "display", "none");
			}
			this._fileInput = document.createElement('input');
			this._fileInput.setAttribute("type","file");
			this.fileInputs.push(this._fileInput);
			// server will need to know this variable:
			var nm = this.fieldName;
			var _id = this.id;
			if(this.selectMultipleFiles){
				nm += this.fileCount;
				_id+=this.fileCount;
				this.fileCount++;
			}
			this._fileInput.setAttribute("id", this.id);
			this._fileInput.setAttribute("name",nm);
			dojo.addClass(this._fileInput,"dijitFileInputReal");
			this._formNode.appendChild(this._fileInput);
			
		},
		
		_removeFileInput: function(){
			dojo.forEach(this.fileInputs, function(inp){
				inp.parentNode.removeChild(inp);									   
			});
			this.fileInputs = [];
			this.fileCount = 0;
		},
		
		_buildForm: function(domNode){
			// summary
			//	Build the form that holds the fileInput
			//	This form also holds the class that targets
			//	the input to change its size
			//
			if(this._formNode) return;

			if(dojo.isIE){
				// just to reiterate, IE is a steaming pile of code. 
				this._formNode = document.createElement('<form enctype="multipart/form-data" method="post">');
				this._formNode.encoding = "multipart/form-data";
				
			}else{
				// this is how all other sane browsers do it
				this._formNode = document.createElement('form');
				this._formNode.setAttribute("enctype","multipart/form-data");
			}
			this._formNode.id = dijit.getUniqueId("form");
			if (domNode && dojo.style(domNode, "display").indexOf("inline") > -1) {
				document.body.appendChild(this._formNode);
			}else{
				domNode.appendChild(this._formNode);
			}
			this._setFormStyle();
		},
		
		_connectInput: function(){
			this._disconnectInput();
			this._cons.push(dojo.connect(this._fileInput, "mouseover", this, function(evt){
				this.onMouseOver(evt);
			}));
			this._cons.push(dojo.connect(this._fileInput, "mouseout", this, function(evt){
				this.onMouseOut(evt);
			}));
			this._cons.push(dojo.connect(this._fileInput, "change", this, function(){
				this.onChange([{name:this._fileInput.value, type:"", size:0}]);
			}));
			
			this._cons.push(dojo.connect(window, "resize", this, "setPosition"));
		},
		
		_disconnectInput: function(){
			dojo.forEach(this._cons, function(c){
				dojo.disconnect(c);									  
			});
		},
		
		_setFormStyle: function(){
			// summary
			//	Apply a dynamic style to the form and input
			//
			// YAY! IE makes us jump through more hoops!
			//	We want to make the fileInput's button large enough to cover our
			//	fake button, and we do this with fontSize=(x)em.
			// 	It seems that after you build a fileInput, it's too late to style it. IE 
			//	styles the input field, but not the button. 
			//	To style the button, we'll create a class that fits, apply it to the form, 
			//	then it will cascade down properly. Geez.
			//
			// If the fake button is bigger than the fileInput, we need to resize
			//	the fileInput. Due to browser security, the only consistent sizing 
			//	method is font EMs. We're using a rough formula here to determine 
			//	if the fake button is very tall or very wide, and resizing based
			//	on the result.
			// We want a minimum of 2em, because on a Mac, system buttons have 
			//	rounded corners. The larger size moves that corner out of position
			var fake = this._getFakeButtonSize();
			var size = Math.max(2,Math.max(Math.ceil(fake.w/60),Math.ceil(fake.h/15)));
			
			// Now create a style associated with the form ID
			dojox.html.insertCssRule("#"+this._formNode.id+" input", "font-size:"+size+"em");
		},
		
		onMouseOver: function(evt){
			// Can be connected to for manipulating hover state
			if(this.button.domNode){
				dojo.addClass(this.button.domNode, "dijitButtonHover dijitHover");
			}
		},
		
		onMouseOut: function(evt){
			// Can be connected to for manipulating hover state
			if(this.button.domNode){
				dojo.removeClass(this.button.domNode, "dijitButtonHover dijitHover");
			}
		},
		
		destroyAll: function(){
			// Destroys everything including button
			this.button.destroy();
			this.destroy();
		},
		
		destroy: function(){
			// summary
			//	Destroys the FileInputOverlay
			//	NOTE: Does not destroy the button or node to which it was 
			//	"attached". That will need to be destroyed seperately.
			this._disconnectInput();
			dojo._destroyElement(this._formNode);
		}
		
	}
);