dojo.require("dojox.form.FileUploader");
dojo.require("dijit.form.Button"); 
dojo.require("dojo.parser");

//using this early for the forceNoFlash test:
dojo.require("dojox.embed.Flash");

var passthrough = function(msg){
	//for catching messages from Flash
	if(window.console){
		console.log(msg);	
	}
}

forceNoFlash = false;
selectMultipleFiles = false;

var qs = window.location.href.split("?");

if(qs.length>1){
	qs = qs[1];
	if(qs.indexOf("forceNoFlash")>-1){
		forceNoFlash = true;
	}
	if(qs.indexOf("multiMode")>-1){
		selectMultipleFiles = true;
	}
}
 
var setLoc = function(href){
	window.location.href = window.location.href.split("?")[0] + href;
}

var showWithFlash = function(){
	if(forceNoFlash){
		setLoc("");
	}
}

var showWithoutFlash = function(){
	if(!forceNoFlash){
		setLoc((selectMultipleFiles) ? "?forceNoFlash&multiMode" : "?forceNoFlash"); 
	}
}

var showMulti = function(){
 	if(!selectMultipleFiles){
		setLoc((forceNoFlash) ? "?forceNoFlash&multiMode" : "?multiMode");
	}
}

var showSingle = function(){
 	if(selectMultipleFiles){
		setLoc((forceNoFlash) ? "?forceNoFlash" : ""); 
	}
}

var imageHTML = function(data){
	console.log("DATA:", data);
	var w = (data.width<320)?data.width:320;
	if(data.creationDate){
		var d = data.creationDate.toString().split(" ");
		console.log("D:", d)
		var date = d[0]+" "+d[1]+" "+d[2]+" "+d[3];
	}else{
		date = "NA";	
	}
	console.log("DATE:",date);
	var name = data.file.split("/")[data.file.split("/").length-1];
	var txt = 	'<div class="picFrame">'+
    			'<img src="'+data.file+'" width="'+w+'">'+
				'<div class="picDesc"'+
					'<div class="name"><strong>'+name+'</strong></div>'+
					'<div class="date">Date Created: <strong>'+date+'</strong></div>'+
					'<div class="dim">Dimensions: <strong>'+data.width+' x '+data.height+'</strong></div>'+
					'<div class="size">Size: <strong>'+Math.ceil(data.size*.001)+'KB</strong></div>'+
					
				'</div>'+
				'</div>'
	return txt;
}
var uploadUrl = "UploadFile.php";
var rmFiles = "";
var fileMask = [
	["Jpeg File", 	"*.jpg;*.jpeg"],
	["GIF File", 	"*.gif"],
	["PNG File", 	"*.png"],
	["All Images", 	"*.jpg;*.jpeg;*.gif;*.png"]
];
// For testing 1D array masks:
// var fileMask = 	["All Images", 	"*.jpg;*.jpeg;*.gif;*.png"];
// var fileMask = 	["PNG File", 	"*.png"];

dojo.addOnLoad(function(){

	if(forceNoFlash){
		dojox.embed.Flash.available = 0;
		dojo.byId("hasFlash").style.display = "none";
		dojo.byId("fTypes").style.display = "none";
	}else{
		dojo.byId("noFlash").style.display = "none";
		if(dojo.isArray(fileMask[0])){
			dojo.byId("fTypes").innerHTML+=fileMask[fileMask.length-1][1];
		}else{
			dojo.byId("fTypes").innerHTML+=fileMask[1];
		}
	}
	
	if(selectMultipleFiles){
		dojo.byId("fmode").innerHTML = dojo.byId("hmode").innerHTML = "Multi-File Mode";
		dojo.byId("fSingle").style.display = "none";
		dojo.byId("hSingle").style.display = "none";
		dijit.byId("fbm").domNode.style.display = "none";
		dijit.byId("hbm").domNode.style.display = "none";
	}else{
		dojo.byId("fmode").innerHTML = dojo.byId("hmode").innerHTML = "Single-File Mode";
		dojo.byId("fMulti").style.display = "none";
		dojo.byId("hMulti").style.display = "none";
		dijit.byId("fbs").domNode.style.display = "none";
		dijit.byId("hbs").domNode.style.display = "none";
	}
	dojo.byId("uploadedFiles").value = "";
	dojo.byId("fileToUpload").value = "";

	console.log("LOC:", window.location)
	console.log("UPLOAD URL:",uploadUrl);
	var f0 = new dojox.form.FileUploader({
		button:dijit.byId("btn0"), 
		degradable:true,
		uploadUrl:uploadUrl, 
		uploadOnChange:false, 
		selectMultipleFiles:selectMultipleFiles,
		fileMask:fileMask,
		isDebug:true
	});
	
	doUpload = function(){
		console.log("doUpload")
		dojo.byId("fileToUpload").innerHTML = "uploading...";
		f0.upload();
	}
	dojo.connect(f0, "onChange", function(data){
		console.log("DATA:", data);
		dojo.forEach(data, function(d){
			//file.type no workie from flash selection (Mac?)
			if(selectMultipleFiles){
				dojo.byId("fileToUpload").value += d.name+" "+Math.ceil(d.size*.001)+"kb \n";
			}else{
				dojo.byId("fileToUpload").value = d.name+" "+Math.ceil(d.size*.001)+"kb \n";
			}
		});
	});

	dojo.connect(f0, "onProgress", function(data){
		console.warn("onProgress", data);
		dojo.byId("fileToUpload").value = "";
		dojo.forEach(data, function(d){
			dojo.byId("fileToUpload").value += "("+d.percent+"%) "+d.name+" \n";
			
		});
	});

	dojo.connect(f0, "onComplete", function(data){
		console.warn("onComplete", data);
		dojo.forEach(data, function(d){
			dojo.byId("uploadedFiles").value += d.file+" \n";
			dojo.byId("rgtCol").innerHTML += imageHTML(d);//'<img src="'+d.file+'" />';
			rmFiles+=d.file+";";
		});
	});
	
	Destroy = function(){
		f0.destroyAll();
	}
	
});

var cleanUp = function(){
	dojo.byId("rgtCol").innerHTML = "";
	dojo.byId("uploadedFiles").value = "";
	dojo.byId("fileToUpload").value = "";
	dojo.xhrGet({
		url:uploadUrl,
		handleAs:"text",
		content:{
			rmFiles:rmFiles
		}
	});
	rmFiles = "";
	
}

dojo.addOnUnload(function(){
	console.log("You're leaving the page");
	cleanUp();
});
