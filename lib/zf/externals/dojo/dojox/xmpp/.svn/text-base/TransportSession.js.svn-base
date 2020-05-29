dojo.provide("dojox.xmpp.TransportSession");
dojo.require("dojox.xmpp.util");
dojo.require("dojo.io.script");
dojo.require("dojo.io.iframe");
dojo.require("dojox.data.dom");

dojox.xmpp.TransportSession = function(props) {
	//mixin any options that we want to provide to this service
	if (props && dojo.isObject(props)) {
		dojo.mixin(this, props);
		if(this.useScriptSrcTransport){
			this.transportIframes = [];
		
		}
	}
	
};


dojox.xmpp.TransportSession._iframeOnload = function(index) {
	//console.log("Loaded transport iframe slot " + index, this);
	var doc = dojo.io.iframe.doc(dojo.byId("xmpp-transport-" + index));
	//console.log("iframe document ", doc);
	doc.write("<script>var isLoaded=true; var rid=0; var transmiting=false; function _BOSH_(msg) { transmiting=false; parent.dojox.xmpp.TransportSession.handleBOSH(msg, rid); } </script>");
};


dojox.xmpp.TransportSession.handleBOSH = function(msg, rid) {
	//console.log("BOSH from iframe", msg, rid);
};


dojo.extend(dojox.xmpp.TransportSession, {

		/* options/defaults */
		rid: 0,
		hold: 1,
		polling:1000,
		secure: false,
		wait: 60,
		lang: 'en',
		submitContentType: 'text/xml; charset=utf=8',
		serviceUrl: '/httpbind',
		defaultResource: "dojoIm",
		domain: 'imserver.com',
		sendTimeout: (this.wait+20)*1000,
		
		useScriptSrcTransport:false,
		
		
		keepAliveTimer:null,

		//status
		state: "NotReady",
		transmitState: "Idle",

		protocolPacketQueue: [],
		outboundQueue: [],
		outboundRequests: {},
		inboundQueue: [],
		deferredRequests: {},
		matchTypeIdAttribute: {},
		
		open: function() {
			this.status = "notReady";
			this.rid = Math.round(Math.random() * 1000000000);
			this.protocolPacketQueue = [];
			this.outboundQueue = [];
			this.outboundRequests = {};
			this.inboundQueue = [];
			this.deferredRequests = {};
			this.matchTypeIdAttribute = {};
			
		
			this.keepAliveTimer = setTimeout(dojo.hitch(this, "_keepAlive"), 10000);
			
			if(this.useScriptSrcTransport){
				dojo.connect(dojox.xmpp.TransportSession, 
					"handleBOSH", this, "processScriptSrc");
				
				this.transportIframes = [];
				
				for(var i = 0; i <= this.hold; i++) {
					var iframe = dojo.io.iframe.create("xmpp-transport-" + i, dojox._scopeName + ".xmpp.TransportSession._iframeOnload("+i+");" );
					this.transportIframes.push(iframe);
					if(i ==0) {
						dojo.connect(iframe, "onload",  this, "_sendLogin");
					}
				}
				
			
			} else {
				this._sendLogin();
			}
		},
		
		_sendLogin: function() {
				var rid = this.rid++;
				var req = {
					content: this.submitContentType,
					hold: this.hold,
					rid: rid,
					to: this.domain,
					secure: this.secure,
					wait: this.wait,
					"xml:lang": this.lang,
					xmlns: dojox.xmpp.xmpp.BODY_NS
				};

				var msg = dojox.xmpp.util.createElement("body", req, true);
				this.addToOutboundQueue(msg, rid);
		},
		
		processScriptSrc: function(msg, rid) {
			//console.log("processScriptSrc::", rid, msg);
		//	var msgDom = dojox.xml.DomParser.parse(msg);
			var msgDom = dojox.data.dom.createDocument(msg, "text/xml");
			//console.log("parsed mgs", msgDom);
			//console.log("Queue", this.outboundQueue);
			if(msgDom) {
				this.processDocument(msgDom, rid);
			} else {
				//console.log("Recived bad document from server",msg);
			}
		},
		
		_keepAlive: function(){
			if (this.state=="wait" || this.isTerminated()) {
				return;
			}
			this._dispatchPacket();
			this.keepAliveTimer = setTimeout(dojo.hitch(this, "_keepAlive"), 10000);
		},
		
		
		close: function(protocolMsg){

	
			var rid = this.rid++;
			var req = {
				
				sid: this.sid,
				rid: rid,
				type: "terminate"
			};
			var envelope = null;

			if (protocolMsg) {
				envelope = new dojox.string.Builder(dojox.xmpp.util.createElement("body", req, false));
				envelope.append(protocolMsg);
				envelope.append("</body>");
			} else {
				envelope = new dojox.string.Builder(dojox.xmpp.util.createElement("body", req, false));
			}

		//	this.sendXml(envelope,rid);
			this.addToOutboundQueue(envelope.toString(), rid);
			this.state=="Terminate";
		},

		dispatchPacket: function(msg, protocolMatchType, matchId, matchProperty){
			// summary
			// Main Packet dispatcher, most calls should be made with this other 
			// than a few setup calls which use add items to the queue directly 
			//protocolMatchType, matchId, and matchProperty are optional params
			//that allow a deferred to be tied to a protocol response instad of the whole
			//rid
	
		//	//console.log("In dispatchPacket ", msg, protocolMatchType, matchId, matchProperty);
			if (msg){
				this.protocolPacketQueue.push(msg);	
			}
			
			var def = new dojo.Deferred();
			//def.rid = req.rid;

			if (protocolMatchType && matchId){
				def.protocolMatchType = protocolMatchType;
				def.matchId = matchId;	
				def.matchProperty = matchProperty || "id";	
				if(def.matchProperty != "id") {
					this.matchTypeIdAttribute[protocolMatchType] = def.matchProperty;
				}
			}

			this.deferredRequests[def.protocolMatchType + "-" +def.matchId]=def;
			if(!this.dispatchTimer) {
				this.dispatchTimer = setTimeout(dojo.hitch(this, "_dispatchPacket"), 600);
			}
			return def;	
		},
	
		_dispatchPacket: function(){
			
			clearTimeout(this.dispatchTimer);
			delete this.dispatchTimer;
			
			if (!this.sid){
				console.debug("TransportSession::dispatchPacket() No SID, packet dropped.")
				return;
			}

			if (!this.authId){
				//FIXME according to original nodes, this should wait a little while and try
				//      again up to three times to see if we get this data.
				console.debug("TransportSession::dispatchPacket() No authId, packet dropped [FIXME]")
				return;
			}

		

			//if there is a pending request with the server, don't poll
			if (this.transmitState != "error" && (this.protocolPacketQueue.length == 0) && (this.outboundQueue.length > 0)) {
				return;
			}

			if (this.state=="wait" || this.isTerminated()) {
				return;
			}

			var req = {
				sid: this.sid
			}

			if (this.protocolPacketQueue.length > 0){
				req.rid= this.rid++;
				var envelope = new dojox.string.Builder(dojox.xmpp.util.createElement("body", req, false));
				envelope.append(this.processProtocolPacketQueue());
				envelope.append("</body>");
				delete this.lastPollTime;
			} else {
				//console.log("Nothing to send, I'm just polling.");
				if(this.lastPollTime) {
					var now = new Date().getTime();
					if(now - this.lastPollTime < this.polling) {
						//console.log("Waiting to poll ", this.polling - (now - this.lastPollTime)+10);
						this.dispatchTimer = setTimeout(dojo.hitch(this, "_dispatchPacket"), this.polling - (now - this.lastPollTime)+10);
						return;
					}
				
				} 
				req.rid= this.rid++;
				this.lastPollTime = new Date().getTime();
				var envelope = new dojox.string.Builder(dojox.xmpp.util.createElement("body", req, true));

			}

		
			this.addToOutboundQueue(envelope.toString(),req.rid);

		},

		redispatchPacket: function(rid){
			var env = this.outboundRequests[rid];
			this.sendXml(env, rid);
		},

		addToOutboundQueue: function(msg, rid){
			this.outboundQueue.push({msg: msg,rid: rid});
			this.outboundRequests[rid]=msg;
			this.sendXml(msg, rid);
		},

		removeFromOutboundQueue: function(rid){
			for(var i=0; i<this.outboundQueue.length;i++){
				if (rid == this.outboundQueue[i]["rid"]){
					this.outboundQueue.splice(i, 1);
					break;
				}
			}
			delete this.outboundRequests[rid];
		},

		processProtocolPacketQueue: function(){
			var packets = new dojox.string.Builder();
			for(var i=0; i<this.protocolPacketQueue.length;i++){
				packets.append(this.protocolPacketQueue[i]);
			}
			this.protocolPacketQueue=[];
			return packets.toString();
		},

		findOpenIframe: function() {
			for(var i = 0; i < this.transportIframes.length; i++) {
				var iframe = this.transportIframes[i];
				var win = iframe.contentWindow;
				//console.log("Open transport?", win, win.isLoaded, win.transmiting);
				
				if(win.isLoaded && !win.transmiting) {
					return iframe;
				}
			
			}
		},
		
		sendXml: function(message, rid){
				if(this.isTerminated()) {
					return;
				}
			//console.log("TransportSession::sendXml()"+ new Date().getTime() + " RID: ", rid, " MSG: ", message);
			this.transmitState = "transmitting";
			if(this.useScriptSrcTransport) { 
				//console.log("using script src to transmit");
			
				var iframe = this.findOpenIframe();
				var iframeDoc = dojo.io.iframe.doc(iframe);
				iframe.contentWindow.rid=rid;
				iframe.contentWindow.transmiting=true;
				dojo.io.script.attach("rid-"+rid,this.serviceUrl+"?"+encodeURIComponent(message),iframeDoc);
								
			} else {
				var def = dojo.rawXhrPost({
					contentType: "text/xml",
					url: this.serviceUrl,
					postData: message,
					handleAs: "xml",
					error: dojo.hitch(this, function(res, io) {
						////console.log("foo", res, io.xhr.responseXML, io.xhr.status);
						return this.processError(io.xhr.responseXML, io.xhr.status , rid);
					}
					),
					timeout: this.sendTimeout
				});

				//process the result document 
				def.addCallback(this, function(res){
					return this.processDocument(res, rid);
				});
				return def;
			}

		},

		processDocument: function(doc, rid){
			if(this.isTerminated()) {
				return;
			}
			//console.log("TransportSession:processDocument() ", doc, rid);
			this.transmitState = "idle";

			var body = doc.firstChild;
			if (body.nodeName != 'body'){
				//console.log("TransportSession::processDocument() firstChild is not <body> element ", doc, " RID: ", rid);
			}

			if (this.outboundQueue.length<1){return;}

			var expectedId = this.outboundQueue[0]["rid"];
			//console.log("expectedId", expectedId);
			if (rid==expectedId){
				this.removeFromOutboundQueue(rid);	
				this.processResponse(body, rid);
				this.processInboundQueue();	
			}else{
				//console.log("TransportSession::processDocument() rid: ", rid, " expected: ", expectedId);
				var gap = rid-expectedId;
			
				if (gap < this.hold + 2){
					this.addToInboundQueue(doc,rid);	
				}else{
					//console.log("TransportSession::processDocument() RID is outside of the expected response window");
				}
			}
			return doc;	
		},

		processInboundQueue: function(){
			while (this.inboundQueue.length > 0) {
				var item = this.inboundQueue.shift();
				this.processDocument(item["doc"], item["rid"]);
			}
		},

		addToInboundQueue: function(doc,rid){
			for (var i=0; i<this.inboundQueue.length;i++){
				if (rid < this.inboundQueue[i]["rid"]){continue;}
				this.inboundQueue.splice(i,0,{doc: doc, rid: rid});	
			}
		},

		processResponse: function(body,rid){
			////console.log("TransportSession:processResponse() ", body, " RID: ", rid);

			if (body.getAttribute("type")=='terminate'){
				var reasonNode = body.firstChild.firstChild;
				var errorMessage = "";
					if(reasonNode.nodeName == "conflict") {
						errorMessage = "conflict"
					}
				this.setState("Terminate", errorMessage);
	
				return;
			}

			if ((this.state != 'Ready')&&(this.state != 'Terminate')) {
				var sid=body.getAttribute("sid");
				if (sid){
					this.sid=sid;
				} else {
					throw new Error("No sid returned during xmpp session startup");
				}

				this.authId = body.getAttribute("authid");	
				if (this.authId == "") {
					if (this.authRetries-- < 1) {
						console.error("Unable to obtain Authorization ID");
						this.terminateSession();	
					}
				}
				this.wait= body.getAttribute("wait");	
				if( body.getAttribute("polling")){
					this.polling= parseInt(body.getAttribute("polling"))*1000;
				}
			
				//console.log("Polling value ", this.polling);
				this.inactivity = body.getAttribute("inactivity");	
				this.setState("Ready");
			}

			dojo.forEach(body.childNodes, function(node){
				this.processProtocolResponse(node, rid);
			}, this);

			//need to make sure, since if you use sendXml directly instead of using
			//dispatch packets, there wont' be a call back function here
			//normally the deferred will get fired by a child message at the protocol level
			//but if it hasn't fired by now, go ahead and fire it with the full body
			/*if (this.deferredRequests[rid] && this.deferredRequests[rid].fired==-1){
				this.deferredRequests[rid].callback(body);
			}*/

			//delete from the list of outstanding requests
			//delete this.deferredRequests[rid];

			if (this.transmitState == "idle"){
				this.dispatchPacket();
			}
		},


		processProtocolResponse: function(msg, rid){
			//summary
			//process the individual protocol messages and if there
			//is a matching set of protocolMatchType, matchId, and matchPropery
			//fire off the deferred

			this.onProcessProtocolResponse(msg);
			var key = msg.nodeName + "-" +msg.getAttribute("id");
			var def = this.deferredRequests[key];
			if (def){
				def.callback(msg);
				delete this.deferredRequests[key];
			}
		},

		setState: function(state, message){
			if (this.state != state) {
				if (this["on"+state]){
					this["on"+state](state, this.state, message);
				}
				this.state=state;
			}
		},
		
		isTerminated: function() {
			
			return this.state=="Terminate";
		},

		processError: function(err, httpStatusCode,rid){
			//console.log("Processing server error ", err, httpStatusCode,rid);
			if(this.isTerminated()) {
				return;
			}
			
			
			if(httpStatusCode != 200) {
				this.setState("Terminate", errorMessage);
				return;
			}
			
			if (err && err.dojoType && err.dojoType=="timeout"){
				//console.log("Wait timeout");
			}
			
			this.removeFromOutboundQueue(rid);	
			//FIXME conditional processing if request will be needed based on type of error.
			if(err && err.firstChild) {
			//console.log("Error ", err.firstChild.getAttribute("type") + " status code " + httpStatusCode);
			
				if (err.firstChild.getAttribute("type")=='terminate'){
					var reasonNode = err.firstChild.firstChild;
					var errorMessage = "";
					if(reasonNode && reasonNode.nodeName == "conflict") {
						errorMessage = "conflict"
					}
					this.setState("Terminate", errorMessage);
					return;
				}			
			}
			this.transmitState = "error";
			setTimeout(dojo.hitch(this, function(){ this.dispatchPacket(); }), 200);	
			//console.log("Error: ", arguments);
			return true;
		},

		//events
		onTerminate: function(newState, oldState, message){ },
		onProcessProtocolResponse: function(msg){},
		onReady: function(newState, oldState){}
});
