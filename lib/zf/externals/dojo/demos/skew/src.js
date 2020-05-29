dojo.provide("demos.skew.src");
	
dojo.require("demos.skew.src.Image");
	
dojo.require("dojo.NodeList-fx");
dojo.require("dojo.fx.easing");
dojo.require("dojox.widget.Dialog");
dojo.require("dojox.layout.RadioGroup");

(function($){
	
	var _stuffMoving = false;
	var _started = false;
	var _stalltime = 650; // ms
	var _doneAnim = null;
	var _needed = null;
	var _loaded = [];
	var _connects = [];
	var profileWidget = null;
	var stack = null;
	var timer = null;
	var _profiletimer = null;
	
	var _doneAnim = function(){
		
		$.fadeOut({ 
			// fade out the overlay
			node: "overlay", 
			duration: 300,
			onEnd: function(){
				// then make our image surface 240px, and fade it in
				$.animateProperty({
					node:"imageContainer", 
					properties: {
						height: 240,
						opacity:1
					},
					// with teh sexy
					easing: $.fx.easing.backOut,
					duration:420,
					// wait for the fadeout above to finish
					delay:600,
					onEnd: function(){
						// hide the actual overlay so you can click underneath it
						$.style("overlay","display","none");
						_started = true;	
						// take all the elements with "startsHidden" class, and 
						// set them up, fade them in, and remove hidden class
						$.query(".startsHidden")
							.style("opacity", "0")
							.removeClass("startsHidden")
							.fadeIn().play(500);
					}
				}).play(); 
			}
		}).play();
		
	};
	
	// onShow result, it no timer was running (see later)
	var updateProfile = function(datawidget){
		//if(!_started){ return; }
		_stuffMoving = true;
		// conveniently, later, we stored a reference to our avatar (RadioGroupSlide child)
		// in the Image widget. 
		stack.selectChild(datawidget._avatar);
		profileWidget.setData(datawidget._userInfo);
	}

	var _oneBroke = function(n,e){
		_gotOne(n,e);
	}

	// the handler for our image onloader.
	// FIXME: if we get a 404, we'll never finish ...
	var _gotOne = function(n,e){
		_loaded.push(n);
		if(_loaded.length >= $._neededImages){
			$.forEach(_connects, $.disconnect);
			_doneAnim();
		}
	}

	// the onLoad function
	var init = function(){
	
		$.style("whoNode","opacity", 0);
	
		// see, the page "degrades" ;) . This contributor listing page is only a link to
		// http://dojo.jot.com/ContrbutorListing
		$.query("a[href^=http://dojo.jot]").forEach(function(n){ 
			n.parentNode.innerHTML = n.innerHTML; 
		});
	
		// create a "help" dialog out of the node in the page.
		var dialog = new dojox.widget.Dialog({
			dimensions:[640,420]
		},"dialog");
		dialog.startup();
		// setup a way to show it
		$.connect($.byId("helper"), "onclick", dialog, "show");
	
		// set it all off: grab some data from a remote file, and create
		// the interface
		$.xhrGet({
			url:"../resources/users.json",
			handleAs:"json",
			load:function(data){
			
				var labelNode = $.byId("whoNode");
				var _lastNode = null;
				
				profileWidget = new profile.Data({},"profileData");
			
				// create the region where the avatars will live.
				stack = new dojox.layout.RadioGroupSlide({
					style:"width:180px; height:200px",
					// FIXME: when did StackContainer start setting relative explicitly?
					_setupChild: function(/*Widget*/ page){
						$.style(page.domNode,{
							display:"none",
							position:"absolute",
							overflow:"hidden"
						})
						return page; // dijit._Widget
					}						
				},"stack");
		
				// iterate over each of the returned committers, setting up the canvas
				$.forEach(data.committers,function(user, i){
				
					// create an Image in the container, and store the user profile data 
					// in it's instance.
					var im = new image.Skewed({ 
						// use a default square.png if no imgUrl found.
						imgUrl: user.imgUrl || "images/square.png",
						value: user.name
					});
					$.mixin(im,{ _userInfo: user });

					// create a reflection-less scale thumbnail (color) in a div
					var node = $.doc.createElement('div');
					stack.containerNode.appendChild(node);
					node.innerHTML = "<img src='imageReflect.php?spread=0.01&thumbsize=165&src=" + im.imgUrl + "' />";
					
					// and make it a child of our RadioGroupSlide
					var avatar = new dijit.layout.ContentPane({
						id: im.id + "avatar",
						slideFrom:"top"
					}, node);
					
					// mix a reference to the child of the stackContianer in the image widget
					$.mixin(im, { _avatar: avatar });

					// store a ref to our "center" image
					if(i === 0){ _lastNode = im; }
					
					// either add this image to the beginning or append to the end. alternate.
					if(i % 2 == 0){
						$.byId("imageContainer").appendChild(im.domNode);
					}else{
						$.place(im.domNode, "imageContainer", "first");
					}
				
				});
				// this will setup all the children we _just_ added
				stack.startup();
									
				// turn the container holding all the image widgets into the interface
				var ic = new image.Container({
					// for performance (it's a big list/lot of images)
					offOpacity:1, 
					onShow: function(widget){
						// onShow fires _every_ time an image is "centered" visually (no skew)
						// so for UX, we'll defer the "updateProfile()" call until some delay,
						if(timer){ clearTimeout(timer); }
						timer = setTimeout($.partial(updateProfile, widget), _stalltime);
						// but still update one label from our widget data
						labelNode.innerHTML = widget.value + "";
						$.anim("profileArea",{ opacity:0 }, 175); 
					},
					// should be lower, but i want to stage opacity on the edges rather
					// than display:none, so for now this looks better.
					visibleItems: 42,
					// tweak as needed
					spacing:25,
					angle:10
				},"imageContainer");
			
				// find every image in this container, and connect an onload connect to it
				// when each image fires it's onload, the _gotOne function above is alerted
				var _needed = $.query("img", "imageContainer");
				$._neededImages = Number(_needed.length);
				_needed.forEach(function(n,i){
					_connects.push($.connect(n, "onload", $.partial(_gotOne, n)));
					_connects.push($.connect(n, "onerror", $.partial(_oneBroke, n)));
					// if you don't touch the .src attribute _after_ connecting to onload, it 
					// won't fire in weird conditions.
					if($.isIE){ n.src = n.src; }
				});
				
				ic.startup();
				_lastNode.center(); // center the first node (from above creating image widgets)
			
				// resize the image container when the window does, it's fluid 
				$.connect(window, "onresize", ic, "resize");
			
				// make it small, so we can wipe it in
				$.style("imageContainer",{ height:"1px", opacity:0 });
					
			}
		})
	};

	$.declare("profile.Stalker", dijit._Widget, {
		
	});

	$.declare("profile.Data", dijit._Widget, {
		// summary: A simple widget we probably don't even need.
		constructor: function(){
			var $$ = $.byId;
			this.nodeReferences = {
				"location" : $$("proLocation"),
				"website"  : $$("proWebsite"),
				"employer" : $$("proEmployer"),
				"title"    : $$("proRole")
			}
		}, 
		
		setData: function(data){
			var node = this.nodeReferences;
			
			for(var i in node){
				if(i in data){
					$.style(node[i],"display","");
					var txt = (
						i == "website" ? 
							"<a href='" + data[i] + "'>" + data[i] + "</a><br>"   : 
							data[i]
					);
					node[i].innerHTML = txt;
				}else{
					$.style(node[i],"display","none");
				}
			}
			
			// this is really a list of potential aliases, so start clean:
			var nick = "";
			if("handle" in data){
				// add the "handle" attribute
				nick += '"' + data["handle"] + '"';
			}
			if("irc" in data && data["irc"] !== data["handle"]){
				// add the "irc" attribute, if it is different from the "handle" attribute
				nick += ' "' + data["irc"] + '"';
			}
			$.byId("proAliases").innerHTML = nick;
			
			// fade and swipe in the content. 
			// FIXME: weirdness w/ IE7 and no respecting -42 marginLegf
			$.anim("profileArea",{ 
				opacity:{ start:0, end: 0.99 }, 
				paddingLeft:{ start:72, end:0 } }, 520, $.fx.easing.bounceOut
			); 
		}
	});

	var newp = function(){
		// IE6 branch of this demo
		window.location.href = "http://" + (confirm("Hi IE6 user! Is it 2008?") ? "webkit.org" : "mozilla.org") + "/";
	}

	// setup our branch launch: ;) 
	$.addOnLoad(($.isIE < 7 ? newp : init));

})(dojo);
