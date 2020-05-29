dojo.provide("dojox.av.FLVideo");
dojo.experimental("dojox.av.FLVideo");
dojo.require("dijit._Widget");
dojo.require("dojox.embed.Flash");
dojo.require("dojox.av._Media");

dojo.declare("dojox.av.FLVideo", [dijit._Widget, dojox.av._Media], {
			 
	// summary:
	//		Inserts a Flash FLV video into the HTML page and provides methods
	//		and events for controlling the video. Also plays the H264/M4V codec 
	//		with a little trickery: change the '.M4V' extension to '.flv'.
	//
	// example:
	//
	//		markup:
	//		|	<div id="vid" initialVolume=".7", 
	//		|		mediaUrl="../resources/Grog.flv" 
	//		|		dojoType="dojox.av.FLVideo"></div>
	//		programmatic:
	//		|	new dojox.av.FLVideo({
	//		|		initialVolume:.7, 
	//		|		mediaUrl:"../resources/Grog.flv"
	//		|	}, "vid");
	//
	//  mediaUrl: String
	// 		REQUIRED: The Url of the video file that will be played. 
	//		NOTE: Must be either an absolute URL or relative to the HTML file. 
	//		Relative paths will be converted to abslute paths
	//
	// _swfPath: Uri
	//		The path to the video player SWF resource
	_swfPath: dojo.moduleUrl("dojox.av", "resources/video.swf"),
	//
	//
	postCreate: function(){
		// summary:
		// Initialize the media.
		//
		this._subs = [];
		this._cons = [];
		this.mediaUrl = this._normalizeUrl(this.mediaUrl);
		this.initialVolume = this._normalizeVolume(this.initialVolume);	
		
		var args = {
			path:this._swfPath.uri,
			width:"100%",
			height:"100%",
			params:{
				allowFullScreen:true,
				wmode:"transparent"
			},
			// only pass in simple variables - no deep objects
			vars:{
				videoUrl:this.mediaUrl, 
				id:this.id,
				autoPlay:this.autoPlay,
				volume:this.initialVolume,
				isDebug:this.isDebug
			}
		};
		
		// Setting up dojo.subscribes that listens to events
		//	from the player
		this._sub("stageClick",  "onClick");
		this._sub("stageSized",  "onSwfSized");
		this._sub("mediaStatus", "onPlayerStatus");
		this._sub("mediaMeta",   "onMetaData");
		this._sub("mediaError",  "onError");
		this._sub("mediaStart",  "onStart");
		this._sub("mediaEnd",    "onEnd");
	
		this._flashObject = new dojox.embed.Flash(args, this.domNode);
		this._flashObject.onLoad = dojo.hitch(this, function(mov){
			this.flashMedia = mov;
			this.isPlaying = this.autoPlay;
			this.isStopped = !this.autoPlay;
			this.onLoad(this.flashMedia);
			this._initStatus();
			this._update();											 
		});
	},
	
	//  =============================  //
	//  Methods to control the player  //
	//  =============================  //
	
	play: function(/* String? */newUrl){
		// summary:
		//		Plays the video. If an url is passed in, plays the new link.
		this.isPlaying = true;
		this.isStopped = false;
		this.flashMedia.doPlay(this._normalizeUrl(newUrl));
	},
	
	pause: function(){
		// summary:
		// 		Pauses the video
		this.isPlaying = false;
		this.isStopped = false;
		//this.onPause();
		this.flashMedia.pause();
	},
	
	seek: function(/* Float */ time ){
		// summary:
		// 		Goes to the time passed in the argument
		this.flashMedia.seek(time);
	},
	
	
	//  =====================  //
	//  Player Getter/Setters  //
	//  =====================  //
	
	volume: function(/* Float */ vol){
		// summary:
		//		Sets the volume of the video to the time in the
		// argument - between 0 - 1.
		//
		if(vol){
			if(!this.flashMedia) {
				this.initialVolume = vol;	
			}
			this.flashMedia.setVolume(this._normalizeVolume(vol));
		}
		if(!this.flashMedia) {
			return this.initialVolume;
		}
		return this.flashMedia.getVolume(); // Float	
	},
	
	
	
	
	//  ===============  //
	//  Private Methods  //
	//  ===============  //
	
	_checkBuffer: function(/* Float */time, /* Float */bufferLength){
		// summary:
		//		Checks that there is a proper buffer time between
		//		current playhead time and the amount of data loaded.
		//		Works only on FLVs with a duration (not older). Pauses
		//		the video while continuing download.
		//
		if(this.percentDownloaded == 100){
			if(this.isBuffering){
				this.onBuffer(false);
				this.flashMedia.doPlay();
			}
			return;
		}
		
		if(!this.isBuffering && bufferLength<.1){
			this.onBuffer(true);
			this.flashMedia.pause();
			return;
		}
		
		var timePercentLoad = this.percentDownloaded*.01*this.duration;
		
		// check if start buffer needed
		if(!this.isBuffering && time+this.minBufferTime*.001>timePercentLoad){
			this.onBuffer(true);
			this.flashMedia.pause();
		
		// check if end buffer needed
		}else if(this.isBuffering && time+this.bufferTime*.001<=timePercentLoad){
			this.onBuffer(false);
			this.flashMedia.doPlay();
		}
		
	},
	_update: function(){
		// summary:
		//		Helper function to fire onPosition, check download progress,
		//		and check buffer.
		var time = Math.min(this.getTime() || 0, this.duration);
		var dObj = this.flashMedia.getLoaded();
		this.percentDownloaded = Math.ceil(dObj.bytesLoaded/dObj.bytesTotal*100);
		this.onDownloaded(this.percentDownloaded);
		this.onPosition(time);
		if(this.duration){
			this._checkBuffer(time, dObj.buffer);	
		}
		setTimeout(dojo.hitch(this, "_update"), this.updateTime);
	},
	
	_normalizeUrl: function(_url){
		// summary:
		//		Checks that path is relative to HTML file or
		//		convertes it to an absolute path. 
		//
		if(_url && _url.toLowerCase().indexOf("http")<0){
			//
			// Appears to be a relative path. Attempt to  convert it to absolute, 
			// so it will better target the SWF.
			var loc = window.location.href.split("/");
			loc.pop();
			loc = loc.join("/")+"/";
			
			_url = loc+_url;
		}
		return _url;
	},
	_normalizeVolume: function(vol){
		// summary:
		//		Ensures volume is less than one
		//
		if(vol>1){
			while(vol>1){
				vol*=.1	
			}
		}
		return vol;
	},
	_sub: function(topic, method){
		// summary:
		// helper for subscribing to topics
		dojo.subscribe(this.id+"/"+topic, this, method);
	},
	destroy: function(){
		// summary:
		// 		destroys flash
		if(!this.flashMedia){
			this._cons.push(dojo.connect(this, "onLoad", this, "destroy"));	
			return;
		}
		dojo.forEach(this._subs, function(s){
			dojo.unsubscribe(s);								  
		});
		dojo.forEach(this._cons, function(c){
			dojo.disconnect(c);								  
		});
		this._flashObject.destroy();
		//dojo._destroyElement(this.flashDiv);
		
	}
	
});
