dojo.require("dojox.av.FLVideo");
dojo.require("dijit.form.Button");
dojo.require("dijit.form.Slider"); 
dojo.require("dojo.dnd.Source");
dojo.require("dojo.parser");

var passthrough = function(msg){
	//for catching messages from Flash
	if(window.console){
		console.log(msg);	
	}
}

var player, controls, progress, lib, libNode;
var dndItem = {};

var extended = {
	onDropExternal: function(){
		console.warn("DROP --- check to copy, move, or none:", dndItem);
		var node = dojo.byId(dndItem.dragNode.id);
		var target = dndItem.dropNode;
		if(target.id == "videoOverlay") return;
		arguments[2] = true;
		this.inherited("onDropExternal", arguments);
		
	}
};
//dojo.extend(dojo.dnd.Source, extended);
dojo.extend(dojo.dnd.Target, extended);
//dojo.dnd.Target.prototype.onDropExternal = extended.onDropExternal;

createRelated = function(items){
	var txt = '<span class="relText">Related Items:</span>'
	dojo.forEach(items, function(m){
		var id = m.title.replace(/\s/g,"");
		var path = "media/"+id+".flv"; 
		var thumb = "media/thumbs/"+id+".jpg"; 	 
		txt += 	'<div class="related" id="click_'+id+'"><div class="thumb" style="background-image:url(media/thumbs/'+id+'.jpg)"></div>'+
				'<div class="title">'+m.title+'</div>'+
				'<div class="desc">'+m.desc+'</div></div>';
	});
	txt +='';	
	return txt;
}




dojo.addOnLoad(function(){
	libNode = dojo.byId("library"); //dojo.dnd.Source; jsId="dnd_library" within "libContainer"
	lib = dnd_library;

	dojo.xhrGet({
			url:"items.json",
			handleAs:"json",
			load:function(items){
				controls.setItems(items);
				var txt = '';
				dojo.forEach(items, function(m){
					var id = m.id = m.title.replace(/\s/g,"");
					var path = m.path = "media/"+id+".flv"; 
					var thumb = m.thumb = "media/thumbs/"+id+".jpg"; 
					txt += ''+
					'<div class="dojoDndItem" id="dnd_'+id+'" path="'+path+'" dndType="blue">'+
						'<div id="'+id+'" class="thumb" style="background-image:url('+thumb+');"></div>'+
						'<div class="title">'+m.title+'</div>'+
						'<div class="desc">'+m.desc+'</div>'+
					'</div>';						 
				});
				libNode.innerHTML = txt;
				lib.sync();
				
				controls.load(items[0].path);
			}
	});
	
	
	player = new dojox.av.FLVideo({initialVolume:.2, isDebug:false}, "video");
	
	dojo.connect(player, "onLoad", controls, "init");
	dojo.connect(player, "onClick", controls, "toggle");
	dojo.connect(player, "onMetaData", controls.progress, "onMeta");
	dojo.connect(player, "onEnd", controls, "onEnd");
	dojo.connect(player, "onStart", controls, "onStart");
	dojo.connect(player, "onPosition", controls.progress, "onPosition");
	
	dojo.subscribe("/dnd/source/over", function(evt){ 
		//console.log("onDndSourceOver", evt);
		if(evt){
			if(evt.node){
				dndItem.dropNode = evt.node
			}
			if(evt.anchor){
				dndItem.dragNode = evt.anchor
			}
		}
	});
	dojo.subscribe("/dnd/start",  function(evt){ console.log("onDndStart", evt)});
	dojo.subscribe("/dnd/drop",   function(evt){ 
		//console.log( "onDndDrop", evt);
		//console.log("dndItem:", dndItem)
		var node = dojo.byId(dndItem.dragNode.id);
		var target = dndItem.dropNode;
		console.log("TARGET:", target)
		if(target.id=="videoOverlay"){
			var path = dojo.attr(node, "path");
			controls.load(path);
			controls.showPause();
		}
	});
	dojo.subscribe("/dnd/cancel", function(evt){ 
		//console.log( "onDndCancel")
	});
});


controls = {
	progress: {
		init: function(){
			this.duration = null;
			this.seeking = false;
			this.progressBar = dijit.byId("progress");
			dojo.connect(this.progressBar.sliderHandle, "mousedown", this, "startDrag");
			this.timeNode = dojo.byId("timeNode");
			this.durNode = dojo.byId("durNode");
			this.initialized = true;
		},
		onMeta: function(data){
			if(data && data.duration){
				this.duration = data.duration;
				this.durNode.innerHTML = this.timecode(this.duration);
				this.progressBar.attr("disabled", false);
			}else{
				this.duration = null;
				this.durNode.innerHTML = "NA";
				this.progressBar.attr("disabled", true);
				this.progressBar.attr("value", 0);
			}
		},
		onDrag: function(val){
			//
			if(this.seeking){
				//console.log("DRAG:", val);
				var p = val*.01
				player.seek(this.duration*p);
			}
		},
		startDrag: function(){
			this.seeking = true;
			this.conChg = dojo.connect(this.progressBar, "onChange", this, "onDrag");
			this.conUp = dojo.connect(dojo.doc, "mouseup", this, "endDrag")
		},
		endDrag: function(){
			this.seeking = false;
			dojo.disconnect(this.conUp);
			dojo.disconnect(this.conChg);
		},
		onPosition: function(time){
			if(this.initialized){
				this.timeNode.innerHTML = this.timecode(time);
				
				if(this.duration){
					if(!this.seeking){
						// IE freaks if the prgressBar's value goes over 1.0
						var p = Math.min(time/this.duration, 1);
						this.progressBar.attr("value", p*100);
					}
				}
			}
		},
		timecode: function(time){
			ts = time.toString()

			if(ts.indexOf(".")<0){
				ts += ".00"
			}else if(ts.length - ts.indexOf(".")==2){
				ts+="0"
			}else if(ts.length - ts.indexOf(".")>2){
				ts = ts.substring(0, ts.indexOf(".")+3)
			}
			return ts;
		}
	},
	volume: {
		
		init:function(){
			
			this.volNode = dojo.byId("volume");
			this.volBack = dojo.byId("volBack");
			dojo.setSelectable(this.volNode, false);
			dojo.setSelectable(this.volBack, false);
			dojo.setSelectable(dojo.byId("volMask"), false);
			
			this.volDim = dojo.coords(this.volNode);
			var v = player.initialVolume; // returns 0 - 1
			dojo.style(this.volBack, "backgroundPosition", "-"+(this.volDim.w-(this.volDim.w*v))+"px 0px");
			
			dojo.connect(this.volNode, "mousedown", this, "begDrag");
			dojo.connect(dojo.doc, "mouseup", this, "endDrag");
			dojo.connect(this.volNode, "mouseover", this, "over");
			dojo.connect(this.volNode, "mouseout", this, "out");
		},
		onDrag: function(evt){
			var x = evt.clientX - this.volDim.x;
			if(x<0) x = 0;
			if(x>this.volDim.w) x = this.volDim.w;
			var p = x/this.volDim.w;
			player.volume(p);
			var prex = x
			x = Math.ceil(x*.1)*10;
			dojo.style(this.volBack, "backgroundPosition", "-"+(this.volDim.w-x)+"px 0px");
		},
		begDrag: function(){
			this.dragging = true;
			this.conMove = dojo.connect(dojo.doc, "mousemove", this, "onDrag");
		},
		endDrag: function(){
			if(this.conMove) {
				dojo.disconnect(this.conMove);
			}
			this.dragging = false;
			this.out();
		},
		over: function(){
			dojo.addClass(this.volBack, "volBackHover");
		},
		out: function(){
			if(!this.dragging){
				dojo.removeClass(this.volBack, "volBackHover");	
			}
		}
	},
	
	
	init: function(){
		this.progress.init();
		this.volume.init();
		this.initialized = true;
	},
	setItems:function(_items){
		this.items = _items;
	},
	onStart: function(){
		this.hideOverlay();
		this.showPause();
		if(this.conM1) dojo.disconnect(this.conM1);
		if(this.conM2) dojo.disconnect(this.conM1);
		if(this.conTog) dojo.disconnect(this.conTog);
		this.conTog = dojo.connect(dojo.byId("videoOverlay"), "click", this, "toggle");
	},
	toggle: function(){
		if(player.isPlaying){
			player.pause();
		}else{
			player.play();
		}
	},
	onEnd: function(){
		console.log("onEnd")
		dojo.disconnect(this.conTog);
		var rel, m1, m2;
		for(var i=0, len=this.items.length;i<len;i++){
			if(this.items[i].path==this.currentPath){
				m1 = i+1;
				m2 = i+2
				if(m1>len-1){
					m1=0;
					m2=1;
				}else if(m2>len-1){
					m2=0;	
				}
				break;
			}
		}
		m1 = this.items[m1];
		m2 = this.items[m2];
		var txt = createRelated([ m1, m2 ]);
		dojo.byId("relatedNode").innerHTML=txt;
		this.conM1 = dojo.connect(dojo.byId("click_"+m1.id), "click", this, function(){
			this.load(m1.path);																
		});
		this.conM2 = dojo.connect(dojo.byId("click_"+m2.id), "click", this, function(){
			this.load(m2.path);																
		});
		this.showOverlay();
		this.showPlay();
		console.log("onEnd done")
	},
	hideOverlay: function(){
		dojo.style(dojo.byId("relatedContainer"), "display", "none");
		dojo.style(dojo.byId("relatedBack"), "display", "none");
	},
	showOverlay: function(){
		dojo.style(dojo.byId("relatedContainer"), "display", "");
		dojo.style(dojo.byId("relatedBack"), "display", "");
		dojo.fadeIn({node:"relatedContainer", start:0, end:.9, duration:500}).play();
		dojo.fadeIn({node:"relatedBack", start:0, end:.5, duration:500}).play();
	},
	doSlider: function(val){
		console.log("VALUE:", val);
	},
	load: function(path){
		
		if(!this.initialized){
			var c = dojo.connect(this, "init", this, function(){
				this.load(path);
				dojo.disconnect(c);
			});
			return;
		}
		console.log("Play:", path)
		this.currentPath = path;
		player.play(path);
		this.hideOverlay();
		this.showPause();
	},
	doPlay: function(){
		player.play();
		this.hideOverlay();
		this.showPause();
	},
	doPause: function(){
		player.pause();
		this.showPlay();	
	},
	showPlay: function(){
		dojo.style(dijit.byId("pauseButton").domNode, "display", "none");
		dojo.style(dijit.byId("playButton").domNode, "display", "");
	},
	showPause: function(){
		dojo.style(dijit.byId("pauseButton").domNode, "display", "");
		dojo.style(dijit.byId("playButton").domNode, "display", "none");
	}
}