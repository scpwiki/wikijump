dojo.provide("demos.mojo.src.download");
// the code for the "live download" link, adapted from http://dojotoolkit.org/~dante/downloadDojo.html
dojo.require("dojo.io.iframe");
(function(){

	var dojo_ver = "1.2.0";
	
	var node = null;
	var _downloadDialog = {

		node: null,
		closeNode: null,
	
		show: function(e){
			if(!this.node){ this.create(); }
			var anim1 = dojo.animateProperty({
				node: dojo.query("img",this.node)[0],
				duration:1500,
				properties: {
					width: { end: 310, start:1, unit:"px" },
					height: { end: 310, start:1, unit:"px" },
					top: { end:0, start:155, unit:"px" },
					left: { end:0, start:155, unit:"px" }//,
					//paddingTop: { end:1, start:155, unit:"px" }//,
					//opacity: { end: 1, start: 0 }
				},
				easing: dojo.fx.easing.elasticOut
			});
			var anim2 = dojo.fx.slideTo({
				node:this.node,
				top: e.pageY - 55,
				left: e.pageX - 155,
				duration:900,
				easing: dojo.fx.easing.elasticOut
			})
			dojo.fx.combine([anim1,anim2]).play();
			dojo.byId("gravity").disabled = true;
		},

		hide: function(e){
			dojo.byId("gravity").disabled = false;
			e.preventDefault();
			dojo.fx.slideTo({ node:this.node, duration:375, left:-310, top:-50,
				easing:dojo.fx.easing.backIn				
			}).play();
		},

		init: function(e){
			// init the download sequence based on the selected parameters.
			var includeUtils = dojo.byId("build").checked;
			var includeSource = dojo.byId("sourceR").checked;
			var ext = (dojo.byId("tgz").checked ? "tar.gz" : "zip");
			var ver = dojo_ver; 
			
			// make the url:
			var host = "http://download.dojotoolkit.org/";
			var url = host + "release-" + (ver) + "/dojo-release-" + (ver) + (includeSource ? "-src." : ".") + (ext);

			// trigger the save as ... dialog
			dojo.io.iframe.send({
				url: url,
				timeout: 5000
			});
			
			if(includeUtils){
				// and another one if they selected build utils. FIXME: ie7 throws popup warning?
				var utilUrl = host + "release-" + (ver) + "/dojo-release-"+(ver)+"-buildscripts."+(ext);
				setTimeout(function(){
					dojo.io.iframe.send({
						url: utilUrl,
						timeout:5000
					});
				},3000);
			}

		},
		
		create: function(e){
			// dynamically create the dialog box:

			var img = dojo.query("img.clone")[0];
			this.node = dojo.body().appendChild(dojo.doc.createElement('div'));			
			var nimg = this.node.appendChild(dojo.clone(img));
			dojo.style(nimg,"position","absolute");

			var h = this.node.appendChild(dojo.doc.createElement('h1'));
			dojo.addClass(h,"handle");

			this.node.id = "downloadDiv";
			h.innerHTML = "download?";

			var form = this.node.appendChild(dojo.byId("downloadForm"));
			dojo.style(form,"visibility","visible");
			dojo.style(this.node,"zIndex","100");
			new dojo.dnd.Moveable(this.node,{ handle: h });
			
			this.closeNode = dojo.byId("closeNode");
			dojo.connect(this.closeNode,"onclick",this,"hide");
			this.submitNode = dojo.byId("submitNode");
			dojo.connect(this.submitNode,"onclick",this,"init");

		}
	};
	
	var button = null;
	dojo.addOnLoad(function(){
		button = dojo.byId("downloadButton");
		dojo.connect(button,"onclick",_downloadDialog,"show")
	});
	
})();