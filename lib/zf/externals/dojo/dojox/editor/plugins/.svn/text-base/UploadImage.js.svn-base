dojo.provide("dojox.editor.plugins.UploadImage");
dojo.require("dijit._editor._Plugin");
dojo.require("dojox.form.FileUploader");

dojo.experimental("dojox.editor.plugins.UploadImage");

dojo.declare("dojox.editor.plugins.UploadImage",
	dijit._editor._Plugin,
	{
		//summary: 
		// 	Adds an icon to the Editor toolbar that when clicked, opens a system dialog
		//	Although the toolbar icon is a tiny "image" the uploader could be used for 
		//	any file type
		
		tempImageUrl: "",
		iconClassPrefix: "editorIcon",
		useDefaultCommand: false,
		uploadUrl: "",
		fileInput:null,
		
		label:"Mike",
		_initButton: function(){
			this.command = "uploadImage";
			this.editor.commands[this.command] = "Upload Image";
			this.inherited("_initButton", arguments);
			delete this.command;
			setTimeout(dojo.hitch(this, "createFileInput"), 200);
		},
		
		createFileInput: function(){
			var fileMask = [
			["Jpeg File", 	"*.jpg;*.jpeg"],
			["GIF File", 	"*.gif"],
			["PNG File", 	"*.png"],
			["All Images", 	"*.jpg;*.jpeg;*.gif;*.png"]
		];
			console.warn("downloadPath:", this.downloadPath);
			this.fileInput = new dojox.form.FileUploader({isDebug:true,button:this.button, uploadUrl:this.uploadUrl, uploadOnChange:true, selectMultipleFiles:false, fileMask:fileMask});
			
			dojo.connect(this.fileInput, "onChange", this, "insertTempImage");
			dojo.connect(this.fileInput, "onComplete", this, "onComplete");
		},
		
		onComplete: function(data,ioArgs,widgetRef){
			data = data[0];
			// Image is ready to insert
			var tmpImgNode = dojo.withGlobal(this.editor.window, "byId", dojo, [this.currentImageId]);
			var file;
			// download path is mainly used so we can access a PHP script
			// not relative to this file. The server *should* return a qualified path.
			if(this.downloadPath){
				file = this.downloadPath+data.name
			}else{
				file = data.file;
			}
			
			tmpImgNode.src = file;
			if(data.width){
				tmpImgNode.width = data.width;
				tmpImgNode.height = data.height;
			}
		},
		
		insertTempImage: function(){
			// inserting a "busy" image to show something is hapening
			//	during upload and download of the image.
			this.currentImageId = "img_"+(new Date().getTime()); 
			var iTxt = '<img id="'+this.currentImageId+'" src="'+this.tempImageUrl+'" width="32" height="32"/>';
			this.editor.execCommand('inserthtml', iTxt);
		}
		
	}
);

dojo.subscribe(dijit._scopeName + ".Editor.getPlugin",null,function(o){
	if(o.plugin){ return; }
	switch(o.args.name){
	case "uploadImage":
		o.plugin = new dojox.editor.plugins.UploadImage({url: o.args.url});
	}
});
