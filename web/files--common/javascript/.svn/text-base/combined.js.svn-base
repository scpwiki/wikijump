/*
Copyright (c) 2007, Yahoo! Inc. All rights reserved.
Code licensed under the BSD License:
http://developer.yahoo.net/yui/license.txt
version: 2.3.1
*/
if(typeof YAHOO=="undefined"){var YAHOO={};}YAHOO.namespace=function(){var A=arguments,E=null,C,B,D;for(C=0;C<A.length;C=C+1){D=A[C].split(".");E=YAHOO;for(B=(D[0]=="YAHOO")?1:0;B<D.length;B=B+1){E[D[B]]=E[D[B]]||{};E=E[D[B]];}}return E;};YAHOO.log=function(D,A,C){var B=YAHOO.widget.Logger;if(B&&B.log){return B.log(D,A,C);}else{return false;}};YAHOO.register=function(A,E,D){var I=YAHOO.env.modules;if(!I[A]){I[A]={versions:[],builds:[]};}var B=I[A],H=D.version,G=D.build,F=YAHOO.env.listeners;B.name=A;B.version=H;B.build=G;B.versions.push(H);B.builds.push(G);B.mainClass=E;for(var C=0;C<F.length;C=C+1){F[C](B);}if(E){E.VERSION=H;E.BUILD=G;}else{YAHOO.log("mainClass is undefined for module "+A,"warn");}};YAHOO.env=YAHOO.env||{modules:[],listeners:[]};YAHOO.env.getVersion=function(A){return YAHOO.env.modules[A]||null;};YAHOO.env.ua=function(){var C={ie:0,opera:0,gecko:0,webkit:0};var B=navigator.userAgent,A;if((/KHTML/).test(B)){C.webkit=1;}A=B.match(/AppleWebKit\/([^\s]*)/);if(A&&A[1]){C.webkit=parseFloat(A[1]);}if(!C.webkit){A=B.match(/Opera[\s\/]([^\s]*)/);if(A&&A[1]){C.opera=parseFloat(A[1]);}else{A=B.match(/MSIE\s([^;]*)/);if(A&&A[1]){C.ie=parseFloat(A[1]);}else{A=B.match(/Gecko\/([^\s]*)/);if(A){C.gecko=1;A=B.match(/rv:([^\s\)]*)/);if(A&&A[1]){C.gecko=parseFloat(A[1]);}}}}}return C;}();(function(){YAHOO.namespace("util","widget","example");if("undefined"!==typeof YAHOO_config){var B=YAHOO_config.listener,A=YAHOO.env.listeners,D=true,C;if(B){for(C=0;C<A.length;C=C+1){if(A[C]==B){D=false;break;}}if(D){A.push(B);}}}})();YAHOO.lang={isArray:function(B){if(B){var A=YAHOO.lang;return A.isNumber(B.length)&&A.isFunction(B.splice)&&!A.hasOwnProperty(B.length);}return false;},isBoolean:function(A){return typeof A==="boolean";},isFunction:function(A){return typeof A==="function";},isNull:function(A){return A===null;},isNumber:function(A){return typeof A==="number"&&isFinite(A);},isObject:function(A){return(A&&(typeof A==="object"||YAHOO.lang.isFunction(A)))||false;},isString:function(A){return typeof A==="string";},isUndefined:function(A){return typeof A==="undefined";},hasOwnProperty:function(A,B){if(Object.prototype.hasOwnProperty){return A.hasOwnProperty(B);}return !YAHOO.lang.isUndefined(A[B])&&A.constructor.prototype[B]!==A[B];},_IEEnumFix:function(C,B){if(YAHOO.env.ua.ie){var E=["toString","valueOf"],A;for(A=0;A<E.length;A=A+1){var F=E[A],D=B[F];if(YAHOO.lang.isFunction(D)&&D!=Object.prototype[F]){C[F]=D;}}}},extend:function(D,E,C){if(!E||!D){throw new Error("YAHOO.lang.extend failed, please check that all dependencies are included.");}var B=function(){};B.prototype=E.prototype;D.prototype=new B();D.prototype.constructor=D;D.superclass=E.prototype;if(E.prototype.constructor==Object.prototype.constructor){E.prototype.constructor=E;}if(C){for(var A in C){D.prototype[A]=C[A];}YAHOO.lang._IEEnumFix(D.prototype,C);}},augmentObject:function(E,D){if(!D||!E){throw new Error("Absorb failed, verify dependencies.");}var A=arguments,C,F,B=A[2];if(B&&B!==true){for(C=2;C<A.length;C=C+1){E[A[C]]=D[A[C]];}}else{for(F in D){if(B||!E[F]){E[F]=D[F];}}YAHOO.lang._IEEnumFix(E,D);}},augmentProto:function(D,C){if(!C||!D){throw new Error("Augment failed, verify dependencies.");}var A=[D.prototype,C.prototype];for(var B=2;B<arguments.length;B=B+1){A.push(arguments[B]);}YAHOO.lang.augmentObject.apply(this,A);},dump:function(A,G){var C=YAHOO.lang,D,F,I=[],J="{...}",B="f(){...}",H=", ",E=" => ";if(!C.isObject(A)){return A+"";}else{if(A instanceof Date||("nodeType" in A&&"tagName" in A)){return A;}else{if(C.isFunction(A)){return B;}}}G=(C.isNumber(G))?G:3;if(C.isArray(A)){I.push("[");for(D=0,F=A.length;D<F;D=D+1){if(C.isObject(A[D])){I.push((G>0)?C.dump(A[D],G-1):J);}else{I.push(A[D]);}I.push(H);}if(I.length>1){I.pop();}I.push("]");}else{I.push("{");for(D in A){if(C.hasOwnProperty(A,D)){I.push(D+E);if(C.isObject(A[D])){I.push((G>0)?C.dump(A[D],G-1):J);}else{I.push(A[D]);}I.push(H);}}if(I.length>1){I.pop();}I.push("}");}return I.join("");},substitute:function(Q,B,J){var G,F,E,M,N,P,D=YAHOO.lang,L=[],C,H="dump",K=" ",A="{",O="}";for(;;){G=Q.lastIndexOf(A);if(G<0){break;}F=Q.indexOf(O,G);if(G+1>=F){break;}C=Q.substring(G+1,F);M=C;P=null;E=M.indexOf(K);if(E>-1){P=M.substring(E+1);M=M.substring(0,E);}N=B[M];if(J){N=J(M,N,P);}if(D.isObject(N)){if(D.isArray(N)){N=D.dump(N,parseInt(P,10));}else{P=P||"";var I=P.indexOf(H);if(I>-1){P=P.substring(4);}if(N.toString===Object.prototype.toString||I>-1){N=D.dump(N,parseInt(P,10));}else{N=N.toString();}}}else{if(!D.isString(N)&&!D.isNumber(N)){N="~-"+L.length+"-~";L[L.length]=C;}}Q=Q.substring(0,G)+N+Q.substring(F+1);}for(G=L.length-1;G>=0;G=G-1){Q=Q.replace(new RegExp("~-"+G+"-~"),"{"+L[G]+"}","g");}return Q;},trim:function(A){try{return A.replace(/^\s+|\s+$/g,"");}catch(B){return A;}},merge:function(){var C={},A=arguments,B;for(B=0;B<A.length;B=B+1){YAHOO.lang.augmentObject(C,A[B],true);}return C;},isValue:function(B){var A=YAHOO.lang;return(A.isObject(B)||A.isString(B)||A.isNumber(B)||A.isBoolean(B));}};YAHOO.util.Lang=YAHOO.lang;YAHOO.lang.augment=YAHOO.lang.augmentProto;YAHOO.augment=YAHOO.lang.augmentProto;YAHOO.extend=YAHOO.lang.extend;YAHOO.register("yahoo",YAHOO,{version:"2.3.1",build:"541"});/*
Copyright (c) 2007, Yahoo! Inc. All rights reserved.
Code licensed under the BSD License:
http://developer.yahoo.net/yui/license.txt
version: 2.3.1
*/
YAHOO.util.CustomEvent=function(D,B,C,A){this.type=D;this.scope=B||window;this.silent=C;this.signature=A||YAHOO.util.CustomEvent.LIST;this.subscribers=[];if(!this.silent){}var E="_YUICEOnSubscribe";if(D!==E){this.subscribeEvent=new YAHOO.util.CustomEvent(E,this,true);}this.lastError=null;};YAHOO.util.CustomEvent.LIST=0;YAHOO.util.CustomEvent.FLAT=1;YAHOO.util.CustomEvent.prototype={subscribe:function(B,C,A){if(!B){throw new Error("Invalid callback for subscriber to '"+this.type+"'");}if(this.subscribeEvent){this.subscribeEvent.fire(B,C,A);}this.subscribers.push(new YAHOO.util.Subscriber(B,C,A));},unsubscribe:function(D,F){if(!D){return this.unsubscribeAll();}var E=false;for(var B=0,A=this.subscribers.length;B<A;++B){var C=this.subscribers[B];if(C&&C.contains(D,F)){this._delete(B);E=true;}}return E;},fire:function(){var E=this.subscribers.length;if(!E&&this.silent){return true;}var H=[],G=true,D,I=false;for(D=0;D<arguments.length;++D){H.push(arguments[D]);}var A=H.length;if(!this.silent){}for(D=0;D<E;++D){var L=this.subscribers[D];if(!L){I=true;}else{if(!this.silent){}var K=L.getScope(this.scope);if(this.signature==YAHOO.util.CustomEvent.FLAT){var B=null;if(H.length>0){B=H[0];}try{G=L.fn.call(K,B,L.obj);}catch(F){this.lastError=F;}}else{try{G=L.fn.call(K,this.type,H,L.obj);}catch(F){this.lastError=F;}}if(false===G){if(!this.silent){}return false;}}}if(I){var J=[],C=this.subscribers;for(D=0,E=C.length;D<E;D=D+1){J.push(C[D]);}this.subscribers=J;}return true;},unsubscribeAll:function(){for(var B=0,A=this.subscribers.length;B<A;++B){this._delete(A-1-B);}this.subscribers=[];return B;},_delete:function(A){var B=this.subscribers[A];if(B){delete B.fn;delete B.obj;}this.subscribers[A]=null;},toString:function(){return"CustomEvent: '"+this.type+"', scope: "+this.scope;}};YAHOO.util.Subscriber=function(B,C,A){this.fn=B;this.obj=YAHOO.lang.isUndefined(C)?null:C;this.override=A;};YAHOO.util.Subscriber.prototype.getScope=function(A){if(this.override){if(this.override===true){return this.obj;}else{return this.override;}}return A;};YAHOO.util.Subscriber.prototype.contains=function(A,B){if(B){return(this.fn==A&&this.obj==B);}else{return(this.fn==A);}};YAHOO.util.Subscriber.prototype.toString=function(){return"Subscriber { obj: "+this.obj+", override: "+(this.override||"no")+" }";};if(!YAHOO.util.Event){YAHOO.util.Event=function(){var H=false;var J=false;var I=[];var K=[];var G=[];var E=[];var C=0;var F=[];var B=[];var A=0;var D={63232:38,63233:40,63234:37,63235:39};return{POLL_RETRYS:4000,POLL_INTERVAL:10,EL:0,TYPE:1,FN:2,WFN:3,UNLOAD_OBJ:3,ADJ_SCOPE:4,OBJ:5,OVERRIDE:6,lastError:null,isSafari:YAHOO.env.ua.webkit,webkit:YAHOO.env.ua.webkit,isIE:YAHOO.env.ua.ie,_interval:null,startInterval:function(){if(!this._interval){var L=this;var M=function(){L._tryPreloadAttach();};this._interval=setInterval(M,this.POLL_INTERVAL);}},onAvailable:function(N,L,O,M){F.push({id:N,fn:L,obj:O,override:M,checkReady:false});C=this.POLL_RETRYS;this.startInterval();},onDOMReady:function(L,N,M){if(J){setTimeout(function(){var O=window;if(M){if(M===true){O=N;}else{O=M;}}L.call(O,"DOMReady",[],N);},0);}else{this.DOMReadyEvent.subscribe(L,N,M);}},onContentReady:function(N,L,O,M){F.push({id:N,fn:L,obj:O,override:M,checkReady:true});C=this.POLL_RETRYS;this.startInterval();},addListener:function(N,L,W,R,M){if(!W||!W.call){return false;}if(this._isValidCollection(N)){var X=true;for(var S=0,U=N.length;S<U;++S){X=this.on(N[S],L,W,R,M)&&X;}return X;}else{if(YAHOO.lang.isString(N)){var Q=this.getEl(N);if(Q){N=Q;}else{this.onAvailable(N,function(){YAHOO.util.Event.on(N,L,W,R,M);});return true;}}}if(!N){return false;}if("unload"==L&&R!==this){K[K.length]=[N,L,W,R,M];return true;}var Z=N;if(M){if(M===true){Z=R;}else{Z=M;}}var O=function(a){return W.call(Z,YAHOO.util.Event.getEvent(a,N),R);};var Y=[N,L,W,O,Z,R,M];var T=I.length;I[T]=Y;if(this.useLegacyEvent(N,L)){var P=this.getLegacyIndex(N,L);if(P==-1||N!=G[P][0]){P=G.length;B[N.id+L]=P;G[P]=[N,L,N["on"+L]];E[P]=[];N["on"+L]=function(a){YAHOO.util.Event.fireLegacyEvent(YAHOO.util.Event.getEvent(a),P);};}E[P].push(Y);}else{try{this._simpleAdd(N,L,O,false);}catch(V){this.lastError=V;this.removeListener(N,L,W);return false;}}return true;},fireLegacyEvent:function(P,N){var R=true,L,T,S,U,Q;T=E[N];for(var M=0,O=T.length;M<O;++M){S=T[M];if(S&&S[this.WFN]){U=S[this.ADJ_SCOPE];Q=S[this.WFN].call(U,P);R=(R&&Q);}}L=G[N];if(L&&L[2]){L[2](P);}return R;},getLegacyIndex:function(M,N){var L=this.generateId(M)+N;if(typeof B[L]=="undefined"){return -1;}else{return B[L];}},useLegacyEvent:function(M,N){if(this.webkit&&("click"==N||"dblclick"==N)){var L=parseInt(this.webkit,10);if(!isNaN(L)&&L<418){return true;}}return false;},removeListener:function(M,L,U){var P,S,W;if(typeof M=="string"){M=this.getEl(M);}else{if(this._isValidCollection(M)){var V=true;for(P=0,S=M.length;P<S;++P){V=(this.removeListener(M[P],L,U)&&V);}return V;}}if(!U||!U.call){return this.purgeElement(M,false,L);}if("unload"==L){for(P=0,S=K.length;P<S;P++){W=K[P];if(W&&W[0]==M&&W[1]==L&&W[2]==U){K[P]=null;return true;}}return false;}var Q=null;var R=arguments[3];if("undefined"===typeof R){R=this._getCacheIndex(M,L,U);}if(R>=0){Q=I[R];}if(!M||!Q){return false;}if(this.useLegacyEvent(M,L)){var O=this.getLegacyIndex(M,L);var N=E[O];if(N){for(P=0,S=N.length;P<S;++P){W=N[P];if(W&&W[this.EL]==M&&W[this.TYPE]==L&&W[this.FN]==U){N[P]=null;break;}}}}else{try{this._simpleRemove(M,L,Q[this.WFN],false);}catch(T){this.lastError=T;return false;}}delete I[R][this.WFN];delete I[R][this.FN];I[R]=null;return true;},getTarget:function(N,M){var L=N.target||N.srcElement;return this.resolveTextNode(L);},resolveTextNode:function(L){if(L&&3==L.nodeType){return L.parentNode;}else{return L;}},getPageX:function(M){var L=M.pageX;if(!L&&0!==L){L=M.clientX||0;if(this.isIE){L+=this._getScrollLeft();}}return L;},getPageY:function(L){var M=L.pageY;if(!M&&0!==M){M=L.clientY||0;if(this.isIE){M+=this._getScrollTop();}}return M;},getXY:function(L){return[this.getPageX(L),this.getPageY(L)];
},getRelatedTarget:function(M){var L=M.relatedTarget;if(!L){if(M.type=="mouseout"){L=M.toElement;}else{if(M.type=="mouseover"){L=M.fromElement;}}}return this.resolveTextNode(L);},getTime:function(N){if(!N.time){var M=new Date().getTime();try{N.time=M;}catch(L){this.lastError=L;return M;}}return N.time;},stopEvent:function(L){this.stopPropagation(L);this.preventDefault(L);},stopPropagation:function(L){if(L.stopPropagation){L.stopPropagation();}else{L.cancelBubble=true;}},preventDefault:function(L){if(L.preventDefault){L.preventDefault();}else{L.returnValue=false;}},getEvent:function(Q,O){var P=Q||window.event;if(!P){var R=this.getEvent.caller;while(R){P=R.arguments[0];if(P&&Event==P.constructor){break;}R=R.caller;}}if(P&&this.isIE){try{var N=P.srcElement;if(N){var M=N.type;}}catch(L){P.target=O;}}return P;},getCharCode:function(M){var L=M.keyCode||M.charCode||0;if(YAHOO.env.ua.webkit&&(L in D)){L=D[L];}return L;},_getCacheIndex:function(P,Q,O){for(var N=0,M=I.length;N<M;++N){var L=I[N];if(L&&L[this.FN]==O&&L[this.EL]==P&&L[this.TYPE]==Q){return N;}}return -1;},generateId:function(L){var M=L.id;if(!M){M="yuievtautoid-"+A;++A;L.id=M;}return M;},_isValidCollection:function(M){try{return(typeof M!=="string"&&M.length&&!M.tagName&&!M.alert&&typeof M[0]!=="undefined");}catch(L){return false;}},elCache:{},getEl:function(L){return(typeof L==="string")?document.getElementById(L):L;},clearCache:function(){},DOMReadyEvent:new YAHOO.util.CustomEvent("DOMReady",this),_load:function(M){if(!H){H=true;var L=YAHOO.util.Event;L._ready();L._tryPreloadAttach();}},_ready:function(M){if(!J){J=true;var L=YAHOO.util.Event;L.DOMReadyEvent.fire();L._simpleRemove(document,"DOMContentLoaded",L._ready);}},_tryPreloadAttach:function(){if(this.locked){return false;}if(this.isIE){if(!J){this.startInterval();return false;}}this.locked=true;var Q=!H;if(!Q){Q=(C>0);}var P=[];var R=function(T,U){var S=T;if(U.override){if(U.override===true){S=U.obj;}else{S=U.override;}}U.fn.call(S,U.obj);};var M,L,O,N;for(M=0,L=F.length;M<L;++M){O=F[M];if(O&&!O.checkReady){N=this.getEl(O.id);if(N){R(N,O);F[M]=null;}else{P.push(O);}}}for(M=0,L=F.length;M<L;++M){O=F[M];if(O&&O.checkReady){N=this.getEl(O.id);if(N){if(H||N.nextSibling){R(N,O);F[M]=null;}}else{P.push(O);}}}C=(P.length===0)?0:C-1;if(Q){this.startInterval();}else{clearInterval(this._interval);this._interval=null;}this.locked=false;return true;},purgeElement:function(O,P,R){var Q=this.getListeners(O,R),N,L;if(Q){for(N=0,L=Q.length;N<L;++N){var M=Q[N];this.removeListener(O,M.type,M.fn,M.index);}}if(P&&O&&O.childNodes){for(N=0,L=O.childNodes.length;N<L;++N){this.purgeElement(O.childNodes[N],P,R);}}},getListeners:function(N,L){var Q=[],M;if(!L){M=[I,K];}else{if(L=="unload"){M=[K];}else{M=[I];}}for(var P=0;P<M.length;P=P+1){var T=M[P];if(T&&T.length>0){for(var R=0,S=T.length;R<S;++R){var O=T[R];if(O&&O[this.EL]===N&&(!L||L===O[this.TYPE])){Q.push({type:O[this.TYPE],fn:O[this.FN],obj:O[this.OBJ],adjust:O[this.OVERRIDE],scope:O[this.ADJ_SCOPE],index:R});}}}}return(Q.length)?Q:null;},_unload:function(S){var R=YAHOO.util.Event,P,O,M,L,N;for(P=0,L=K.length;P<L;++P){M=K[P];if(M){var Q=window;if(M[R.ADJ_SCOPE]){if(M[R.ADJ_SCOPE]===true){Q=M[R.UNLOAD_OBJ];}else{Q=M[R.ADJ_SCOPE];}}M[R.FN].call(Q,R.getEvent(S,M[R.EL]),M[R.UNLOAD_OBJ]);K[P]=null;M=null;Q=null;}}K=null;if(I&&I.length>0){O=I.length;while(O){N=O-1;M=I[N];if(M){R.removeListener(M[R.EL],M[R.TYPE],M[R.FN],N);}O=O-1;}M=null;R.clearCache();}for(P=0,L=G.length;P<L;++P){G[P][0]=null;G[P]=null;}G=null;R._simpleRemove(window,"unload",R._unload);},_getScrollLeft:function(){return this._getScroll()[1];},_getScrollTop:function(){return this._getScroll()[0];},_getScroll:function(){var L=document.documentElement,M=document.body;if(L&&(L.scrollTop||L.scrollLeft)){return[L.scrollTop,L.scrollLeft];}else{if(M){return[M.scrollTop,M.scrollLeft];}else{return[0,0];}}},regCE:function(){},_simpleAdd:function(){if(window.addEventListener){return function(N,O,M,L){N.addEventListener(O,M,(L));};}else{if(window.attachEvent){return function(N,O,M,L){N.attachEvent("on"+O,M);};}else{return function(){};}}}(),_simpleRemove:function(){if(window.removeEventListener){return function(N,O,M,L){N.removeEventListener(O,M,(L));};}else{if(window.detachEvent){return function(M,N,L){M.detachEvent("on"+N,L);};}else{return function(){};}}}()};}();(function(){var D=YAHOO.util.Event;D.on=D.addListener;if(D.isIE){YAHOO.util.Event.onDOMReady(YAHOO.util.Event._tryPreloadAttach,YAHOO.util.Event,true);var B,E=document,A=E.body;if(("undefined"!==typeof YAHOO_config)&&YAHOO_config.injecting){B=document.createElement("script");var C=E.getElementsByTagName("head")[0]||A;C.insertBefore(B,C.firstChild);}else{E.write("<script id=\"_yui_eu_dr\" defer=\"true\" src=\"//:\"></script>");B=document.getElementById("_yui_eu_dr");}if(B){B.onreadystatechange=function(){if("complete"===this.readyState){this.parentNode.removeChild(this);YAHOO.util.Event._ready();}};}else{}B=null;}else{if(D.webkit){D._drwatch=setInterval(function(){var F=document.readyState;if("loaded"==F||"complete"==F){clearInterval(D._drwatch);D._drwatch=null;D._ready();}},D.POLL_INTERVAL);}else{D._simpleAdd(document,"DOMContentLoaded",D._ready);}}D._simpleAdd(window,"load",D._load);D._simpleAdd(window,"unload",D._unload);D._tryPreloadAttach();})();}YAHOO.util.EventProvider=function(){};YAHOO.util.EventProvider.prototype={__yui_events:null,__yui_subscribers:null,subscribe:function(A,C,F,E){this.__yui_events=this.__yui_events||{};var D=this.__yui_events[A];if(D){D.subscribe(C,F,E);}else{this.__yui_subscribers=this.__yui_subscribers||{};var B=this.__yui_subscribers;if(!B[A]){B[A]=[];}B[A].push({fn:C,obj:F,override:E});}},unsubscribe:function(C,E,G){this.__yui_events=this.__yui_events||{};var A=this.__yui_events;if(C){var F=A[C];if(F){return F.unsubscribe(E,G);}}else{var B=true;for(var D in A){if(YAHOO.lang.hasOwnProperty(A,D)){B=B&&A[D].unsubscribe(E,G);}}return B;}return false;},unsubscribeAll:function(A){return this.unsubscribe(A);},createEvent:function(G,D){this.__yui_events=this.__yui_events||{};
var A=D||{};var I=this.__yui_events;if(I[G]){}else{var H=A.scope||this;var E=(A.silent);var B=new YAHOO.util.CustomEvent(G,H,E,YAHOO.util.CustomEvent.FLAT);I[G]=B;if(A.onSubscribeCallback){B.subscribeEvent.subscribe(A.onSubscribeCallback);}this.__yui_subscribers=this.__yui_subscribers||{};var F=this.__yui_subscribers[G];if(F){for(var C=0;C<F.length;++C){B.subscribe(F[C].fn,F[C].obj,F[C].override);}}}return I[G];},fireEvent:function(E,D,A,C){this.__yui_events=this.__yui_events||{};var G=this.__yui_events[E];if(!G){return null;}var B=[];for(var F=1;F<arguments.length;++F){B.push(arguments[F]);}return G.fire.apply(G,B);},hasEvent:function(A){if(this.__yui_events){if(this.__yui_events[A]){return true;}}return false;}};YAHOO.util.KeyListener=function(A,F,B,C){if(!A){}else{if(!F){}else{if(!B){}}}if(!C){C=YAHOO.util.KeyListener.KEYDOWN;}var D=new YAHOO.util.CustomEvent("keyPressed");this.enabledEvent=new YAHOO.util.CustomEvent("enabled");this.disabledEvent=new YAHOO.util.CustomEvent("disabled");if(typeof A=="string"){A=document.getElementById(A);}if(typeof B=="function"){D.subscribe(B);}else{D.subscribe(B.fn,B.scope,B.correctScope);}function E(K,J){if(!F.shift){F.shift=false;}if(!F.alt){F.alt=false;}if(!F.ctrl){F.ctrl=false;}if(K.shiftKey==F.shift&&K.altKey==F.alt&&K.ctrlKey==F.ctrl){var H;var G;if(F.keys instanceof Array){for(var I=0;I<F.keys.length;I++){H=F.keys[I];if(H==K.charCode){D.fire(K.charCode,K);break;}else{if(H==K.keyCode){D.fire(K.keyCode,K);break;}}}}else{H=F.keys;if(H==K.charCode){D.fire(K.charCode,K);}else{if(H==K.keyCode){D.fire(K.keyCode,K);}}}}}this.enable=function(){if(!this.enabled){YAHOO.util.Event.addListener(A,C,E);this.enabledEvent.fire(F);}this.enabled=true;};this.disable=function(){if(this.enabled){YAHOO.util.Event.removeListener(A,C,E);this.disabledEvent.fire(F);}this.enabled=false;};this.toString=function(){return"KeyListener ["+F.keys+"] "+A.tagName+(A.id?"["+A.id+"]":"");};};YAHOO.util.KeyListener.KEYDOWN="keydown";YAHOO.util.KeyListener.KEYUP="keyup";YAHOO.register("event",YAHOO.util.Event,{version:"2.3.1",build:"541"});/*
Copyright (c) 2007, Yahoo! Inc. All rights reserved.
Code licensed under the BSD License:
http://developer.yahoo.net/yui/license.txt
version: 2.3.1
*/
(function(){var B=YAHOO.util,K,I,H=0,J={},F={};var C=YAHOO.env.ua.opera,L=YAHOO.env.ua.webkit,A=YAHOO.env.ua.gecko,G=YAHOO.env.ua.ie;var E={HYPHEN:/(-[a-z])/i,ROOT_TAG:/^body|html$/i};var M=function(O){if(!E.HYPHEN.test(O)){return O;}if(J[O]){return J[O];}var P=O;while(E.HYPHEN.exec(P)){P=P.replace(RegExp.$1,RegExp.$1.substr(1).toUpperCase());}J[O]=P;return P;};var N=function(P){var O=F[P];if(!O){O=new RegExp("(?:^|\\s+)"+P+"(?:\\s+|$)");F[P]=O;}return O;};if(document.defaultView&&document.defaultView.getComputedStyle){K=function(O,R){var Q=null;if(R=="float"){R="cssFloat";}var P=document.defaultView.getComputedStyle(O,"");if(P){Q=P[M(R)];}return O.style[R]||Q;};}else{if(document.documentElement.currentStyle&&G){K=function(O,Q){switch(M(Q)){case"opacity":var S=100;try{S=O.filters["DXImageTransform.Microsoft.Alpha"].opacity;}catch(R){try{S=O.filters("alpha").opacity;}catch(R){}}return S/100;case"float":Q="styleFloat";default:var P=O.currentStyle?O.currentStyle[Q]:null;return(O.style[Q]||P);}};}else{K=function(O,P){return O.style[P];};}}if(G){I=function(O,P,Q){switch(P){case"opacity":if(YAHOO.lang.isString(O.style.filter)){O.style.filter="alpha(opacity="+Q*100+")";if(!O.currentStyle||!O.currentStyle.hasLayout){O.style.zoom=1;}}break;case"float":P="styleFloat";default:O.style[P]=Q;}};}else{I=function(O,P,Q){if(P=="float"){P="cssFloat";}O.style[P]=Q;};}var D=function(O,P){return O&&O.nodeType==1&&(!P||P(O));};YAHOO.util.Dom={get:function(Q){if(Q&&(Q.tagName||Q.item)){return Q;}if(YAHOO.lang.isString(Q)||!Q){return document.getElementById(Q);}if(Q.length!==undefined){var R=[];for(var P=0,O=Q.length;P<O;++P){R[R.length]=B.Dom.get(Q[P]);}return R;}return Q;},getStyle:function(O,Q){Q=M(Q);var P=function(R){return K(R,Q);};return B.Dom.batch(O,P,B.Dom,true);},setStyle:function(O,Q,R){Q=M(Q);var P=function(S){I(S,Q,R);};B.Dom.batch(O,P,B.Dom,true);},getXY:function(O){var P=function(R){if((R.parentNode===null||R.offsetParent===null||this.getStyle(R,"display")=="none")&&R!=document.body){return false;}var Q=null;var V=[];var S;var T=R.ownerDocument;if(R.getBoundingClientRect){S=R.getBoundingClientRect();return[S.left+B.Dom.getDocumentScrollLeft(R.ownerDocument),S.top+B.Dom.getDocumentScrollTop(R.ownerDocument)];}else{V=[R.offsetLeft,R.offsetTop];Q=R.offsetParent;var U=this.getStyle(R,"position")=="absolute";if(Q!=R){while(Q){V[0]+=Q.offsetLeft;V[1]+=Q.offsetTop;if(L&&!U&&this.getStyle(Q,"position")=="absolute"){U=true;}Q=Q.offsetParent;}}if(L&&U){V[0]-=R.ownerDocument.body.offsetLeft;V[1]-=R.ownerDocument.body.offsetTop;}}Q=R.parentNode;while(Q.tagName&&!E.ROOT_TAG.test(Q.tagName)){if(B.Dom.getStyle(Q,"display").search(/^inline|table-row.*$/i)){V[0]-=Q.scrollLeft;V[1]-=Q.scrollTop;}Q=Q.parentNode;}return V;};return B.Dom.batch(O,P,B.Dom,true);},getX:function(O){var P=function(Q){return B.Dom.getXY(Q)[0];};return B.Dom.batch(O,P,B.Dom,true);},getY:function(O){var P=function(Q){return B.Dom.getXY(Q)[1];};return B.Dom.batch(O,P,B.Dom,true);},setXY:function(O,R,Q){var P=function(U){var T=this.getStyle(U,"position");if(T=="static"){this.setStyle(U,"position","relative");T="relative";}var W=this.getXY(U);if(W===false){return false;}var V=[parseInt(this.getStyle(U,"left"),10),parseInt(this.getStyle(U,"top"),10)];if(isNaN(V[0])){V[0]=(T=="relative")?0:U.offsetLeft;}if(isNaN(V[1])){V[1]=(T=="relative")?0:U.offsetTop;}if(R[0]!==null){U.style.left=R[0]-W[0]+V[0]+"px";}if(R[1]!==null){U.style.top=R[1]-W[1]+V[1]+"px";}if(!Q){var S=this.getXY(U);if((R[0]!==null&&S[0]!=R[0])||(R[1]!==null&&S[1]!=R[1])){this.setXY(U,R,true);}}};B.Dom.batch(O,P,B.Dom,true);},setX:function(P,O){B.Dom.setXY(P,[O,null]);},setY:function(O,P){B.Dom.setXY(O,[null,P]);},getRegion:function(O){var P=function(Q){if((Q.parentNode===null||Q.offsetParent===null||this.getStyle(Q,"display")=="none")&&Q!=document.body){return false;}var R=B.Region.getRegion(Q);return R;};return B.Dom.batch(O,P,B.Dom,true);},getClientWidth:function(){return B.Dom.getViewportWidth();},getClientHeight:function(){return B.Dom.getViewportHeight();},getElementsByClassName:function(S,W,T,U){W=W||"*";T=(T)?B.Dom.get(T):null||document;if(!T){return[];}var P=[],O=T.getElementsByTagName(W),V=N(S);for(var Q=0,R=O.length;Q<R;++Q){if(V.test(O[Q].className)){P[P.length]=O[Q];if(U){U.call(O[Q],O[Q]);}}}return P;},hasClass:function(Q,P){var O=N(P);var R=function(S){return O.test(S.className);};return B.Dom.batch(Q,R,B.Dom,true);},addClass:function(P,O){var Q=function(R){if(this.hasClass(R,O)){return false;}R.className=YAHOO.lang.trim([R.className,O].join(" "));return true;};return B.Dom.batch(P,Q,B.Dom,true);},removeClass:function(Q,P){var O=N(P);var R=function(S){if(!this.hasClass(S,P)){return false;}var T=S.className;S.className=T.replace(O," ");if(this.hasClass(S,P)){this.removeClass(S,P);}S.className=YAHOO.lang.trim(S.className);return true;};return B.Dom.batch(Q,R,B.Dom,true);},replaceClass:function(R,P,O){if(!O||P===O){return false;}var Q=N(P);var S=function(T){if(!this.hasClass(T,P)){this.addClass(T,O);return true;}T.className=T.className.replace(Q," "+O+" ");if(this.hasClass(T,P)){this.replaceClass(T,P,O);}T.className=YAHOO.lang.trim(T.className);return true;};return B.Dom.batch(R,S,B.Dom,true);},generateId:function(O,Q){Q=Q||"yui-gen";var P=function(R){if(R&&R.id){return R.id;}var S=Q+H++;if(R){R.id=S;}return S;};return B.Dom.batch(O,P,B.Dom,true)||P.apply(B.Dom,arguments);},isAncestor:function(P,Q){P=B.Dom.get(P);if(!P||!Q){return false;}var O=function(R){if(P.contains&&R.nodeType&&!L){return P.contains(R);}else{if(P.compareDocumentPosition&&R.nodeType){return !!(P.compareDocumentPosition(R)&16);}else{if(R.nodeType){return !!this.getAncestorBy(R,function(S){return S==P;});}}}return false;};return B.Dom.batch(Q,O,B.Dom,true);},inDocument:function(O){var P=function(Q){if(L){while(Q=Q.parentNode){if(Q==document.documentElement){return true;}}return false;}return this.isAncestor(document.documentElement,Q);};return B.Dom.batch(O,P,B.Dom,true);},getElementsBy:function(V,P,Q,S){P=P||"*";
Q=(Q)?B.Dom.get(Q):null||document;if(!Q){return[];}var R=[],U=Q.getElementsByTagName(P);for(var T=0,O=U.length;T<O;++T){if(V(U[T])){R[R.length]=U[T];if(S){S(U[T]);}}}return R;},batch:function(S,V,U,Q){S=(S&&(S.tagName||S.item))?S:B.Dom.get(S);if(!S||!V){return false;}var R=(Q)?U:window;if(S.tagName||S.length===undefined){return V.call(R,S,U);}var T=[];for(var P=0,O=S.length;P<O;++P){T[T.length]=V.call(R,S[P],U);}return T;},getDocumentHeight:function(){var P=(document.compatMode!="CSS1Compat")?document.body.scrollHeight:document.documentElement.scrollHeight;var O=Math.max(P,B.Dom.getViewportHeight());return O;},getDocumentWidth:function(){var P=(document.compatMode!="CSS1Compat")?document.body.scrollWidth:document.documentElement.scrollWidth;var O=Math.max(P,B.Dom.getViewportWidth());return O;},getViewportHeight:function(){var O=self.innerHeight;var P=document.compatMode;if((P||G)&&!C){O=(P=="CSS1Compat")?document.documentElement.clientHeight:document.body.clientHeight;}return O;},getViewportWidth:function(){var O=self.innerWidth;var P=document.compatMode;if(P||G){O=(P=="CSS1Compat")?document.documentElement.clientWidth:document.body.clientWidth;}return O;},getAncestorBy:function(O,P){while(O=O.parentNode){if(D(O,P)){return O;}}return null;},getAncestorByClassName:function(P,O){P=B.Dom.get(P);if(!P){return null;}var Q=function(R){return B.Dom.hasClass(R,O);};return B.Dom.getAncestorBy(P,Q);},getAncestorByTagName:function(P,O){P=B.Dom.get(P);if(!P){return null;}var Q=function(R){return R.tagName&&R.tagName.toUpperCase()==O.toUpperCase();};return B.Dom.getAncestorBy(P,Q);},getPreviousSiblingBy:function(O,P){while(O){O=O.previousSibling;if(D(O,P)){return O;}}return null;},getPreviousSibling:function(O){O=B.Dom.get(O);if(!O){return null;}return B.Dom.getPreviousSiblingBy(O);},getNextSiblingBy:function(O,P){while(O){O=O.nextSibling;if(D(O,P)){return O;}}return null;},getNextSibling:function(O){O=B.Dom.get(O);if(!O){return null;}return B.Dom.getNextSiblingBy(O);},getFirstChildBy:function(O,Q){var P=(D(O.firstChild,Q))?O.firstChild:null;return P||B.Dom.getNextSiblingBy(O.firstChild,Q);},getFirstChild:function(O,P){O=B.Dom.get(O);if(!O){return null;}return B.Dom.getFirstChildBy(O);},getLastChildBy:function(O,Q){if(!O){return null;}var P=(D(O.lastChild,Q))?O.lastChild:null;return P||B.Dom.getPreviousSiblingBy(O.lastChild,Q);},getLastChild:function(O){O=B.Dom.get(O);return B.Dom.getLastChildBy(O);},getChildrenBy:function(P,R){var Q=B.Dom.getFirstChildBy(P,R);var O=Q?[Q]:[];B.Dom.getNextSiblingBy(Q,function(S){if(!R||R(S)){O[O.length]=S;}return false;});return O;},getChildren:function(O){O=B.Dom.get(O);if(!O){}return B.Dom.getChildrenBy(O);},getDocumentScrollLeft:function(O){O=O||document;return Math.max(O.documentElement.scrollLeft,O.body.scrollLeft);},getDocumentScrollTop:function(O){O=O||document;return Math.max(O.documentElement.scrollTop,O.body.scrollTop);},insertBefore:function(P,O){P=B.Dom.get(P);O=B.Dom.get(O);if(!P||!O||!O.parentNode){return null;}return O.parentNode.insertBefore(P,O);},insertAfter:function(P,O){P=B.Dom.get(P);O=B.Dom.get(O);if(!P||!O||!O.parentNode){return null;}if(O.nextSibling){return O.parentNode.insertBefore(P,O.nextSibling);}else{return O.parentNode.appendChild(P);}}};})();YAHOO.util.Region=function(C,D,A,B){this.top=C;this[1]=C;this.right=D;this.bottom=A;this.left=B;this[0]=B;};YAHOO.util.Region.prototype.contains=function(A){return(A.left>=this.left&&A.right<=this.right&&A.top>=this.top&&A.bottom<=this.bottom);};YAHOO.util.Region.prototype.getArea=function(){return((this.bottom-this.top)*(this.right-this.left));};YAHOO.util.Region.prototype.intersect=function(E){var C=Math.max(this.top,E.top);var D=Math.min(this.right,E.right);var A=Math.min(this.bottom,E.bottom);var B=Math.max(this.left,E.left);if(A>=C&&D>=B){return new YAHOO.util.Region(C,D,A,B);}else{return null;}};YAHOO.util.Region.prototype.union=function(E){var C=Math.min(this.top,E.top);var D=Math.max(this.right,E.right);var A=Math.max(this.bottom,E.bottom);var B=Math.min(this.left,E.left);return new YAHOO.util.Region(C,D,A,B);};YAHOO.util.Region.prototype.toString=function(){return("Region {top: "+this.top+", right: "+this.right+", bottom: "+this.bottom+", left: "+this.left+"}");};YAHOO.util.Region.getRegion=function(D){var F=YAHOO.util.Dom.getXY(D);var C=F[1];var E=F[0]+D.offsetWidth;var A=F[1]+D.offsetHeight;var B=F[0];return new YAHOO.util.Region(C,E,A,B);};YAHOO.util.Point=function(A,B){if(YAHOO.lang.isArray(A)){B=A[1];A=A[0];}this.x=this.right=this.left=this[0]=A;this.y=this.top=this.bottom=this[1]=B;};YAHOO.util.Point.prototype=new YAHOO.util.Region();YAHOO.register("dom",YAHOO.util.Dom,{version:"2.3.1",build:"541"});/*
Copyright (c) 2007, Yahoo! Inc. All rights reserved.
Code licensed under the BSD License:
http://developer.yahoo.net/yui/license.txt
version: 2.3.1
*/
if(!YAHOO.util.DragDropMgr){YAHOO.util.DragDropMgr=function(){var A=YAHOO.util.Event;return{ids:{},handleIds:{},dragCurrent:null,dragOvers:{},deltaX:0,deltaY:0,preventDefault:true,stopPropagation:true,initialized:false,locked:false,interactionInfo:null,init:function(){this.initialized=true;},POINT:0,INTERSECT:1,STRICT_INTERSECT:2,mode:0,_execOnAll:function(D,C){for(var E in this.ids){for(var B in this.ids[E]){var F=this.ids[E][B];if(!this.isTypeOfDD(F)){continue;}F[D].apply(F,C);}}},_onLoad:function(){this.init();A.on(document,"mouseup",this.handleMouseUp,this,true);A.on(document,"mousemove",this.handleMouseMove,this,true);A.on(window,"unload",this._onUnload,this,true);A.on(window,"resize",this._onResize,this,true);},_onResize:function(B){this._execOnAll("resetConstraints",[]);},lock:function(){this.locked=true;},unlock:function(){this.locked=false;},isLocked:function(){return this.locked;},locationCache:{},useCache:true,clickPixelThresh:3,clickTimeThresh:1000,dragThreshMet:false,clickTimeout:null,startX:0,startY:0,regDragDrop:function(C,B){if(!this.initialized){this.init();}if(!this.ids[B]){this.ids[B]={};}this.ids[B][C.id]=C;},removeDDFromGroup:function(D,B){if(!this.ids[B]){this.ids[B]={};}var C=this.ids[B];if(C&&C[D.id]){delete C[D.id];}},_remove:function(C){for(var B in C.groups){if(B&&this.ids[B][C.id]){delete this.ids[B][C.id];}}delete this.handleIds[C.id];},regHandle:function(C,B){if(!this.handleIds[C]){this.handleIds[C]={};}this.handleIds[C][B]=B;},isDragDrop:function(B){return(this.getDDById(B))?true:false;},getRelated:function(G,C){var F=[];for(var E in G.groups){for(var D in this.ids[E]){var B=this.ids[E][D];if(!this.isTypeOfDD(B)){continue;}if(!C||B.isTarget){F[F.length]=B;}}}return F;},isLegalTarget:function(F,E){var C=this.getRelated(F,true);for(var D=0,B=C.length;D<B;++D){if(C[D].id==E.id){return true;}}return false;},isTypeOfDD:function(B){return(B&&B.__ygDragDrop);},isHandle:function(C,B){return(this.handleIds[C]&&this.handleIds[C][B]);},getDDById:function(C){for(var B in this.ids){if(this.ids[B][C]){return this.ids[B][C];}}return null;},handleMouseDown:function(D,C){this.currentTarget=YAHOO.util.Event.getTarget(D);this.dragCurrent=C;var B=C.getEl();this.startX=YAHOO.util.Event.getPageX(D);this.startY=YAHOO.util.Event.getPageY(D);this.deltaX=this.startX-B.offsetLeft;this.deltaY=this.startY-B.offsetTop;this.dragThreshMet=false;this.clickTimeout=setTimeout(function(){var E=YAHOO.util.DDM;E.startDrag(E.startX,E.startY);},this.clickTimeThresh);},startDrag:function(B,D){clearTimeout(this.clickTimeout);var C=this.dragCurrent;if(C){C.b4StartDrag(B,D);}if(C){C.startDrag(B,D);}this.dragThreshMet=true;},handleMouseUp:function(B){if(this.dragCurrent){clearTimeout(this.clickTimeout);if(this.dragThreshMet){this.fireEvents(B,true);}else{}this.stopDrag(B);this.stopEvent(B);}},stopEvent:function(B){if(this.stopPropagation){YAHOO.util.Event.stopPropagation(B);}if(this.preventDefault){YAHOO.util.Event.preventDefault(B);}},stopDrag:function(C,B){if(this.dragCurrent&&!B){if(this.dragThreshMet){this.dragCurrent.b4EndDrag(C);this.dragCurrent.endDrag(C);}this.dragCurrent.onMouseUp(C);}this.dragCurrent=null;this.dragOvers={};},handleMouseMove:function(E){var B=this.dragCurrent;if(B){if(YAHOO.util.Event.isIE&&!E.button){this.stopEvent(E);return this.handleMouseUp(E);}if(!this.dragThreshMet){var D=Math.abs(this.startX-YAHOO.util.Event.getPageX(E));var C=Math.abs(this.startY-YAHOO.util.Event.getPageY(E));if(D>this.clickPixelThresh||C>this.clickPixelThresh){this.startDrag(this.startX,this.startY);}}if(this.dragThreshMet){B.b4Drag(E);if(B){B.onDrag(E);}if(B){this.fireEvents(E,false);}}this.stopEvent(E);}},fireEvents:function(Q,H){var S=this.dragCurrent;if(!S||S.isLocked()){return ;}var J=YAHOO.util.Event.getPageX(Q),I=YAHOO.util.Event.getPageY(Q),K=new YAHOO.util.Point(J,I),F=S.getTargetCoord(K.x,K.y),C=S.getDragEl(),P=new YAHOO.util.Region(F.y,F.x+C.offsetWidth,F.y+C.offsetHeight,F.x),E=[],G=[],B=[],R=[],O=[];for(var M in this.dragOvers){var T=this.dragOvers[M];if(!this.isTypeOfDD(T)){continue;}if(!this.isOverTarget(K,T,this.mode,P)){G.push(T);}E[M]=true;delete this.dragOvers[M];}for(var L in S.groups){if("string"!=typeof L){continue;}for(M in this.ids[L]){var D=this.ids[L][M];if(!this.isTypeOfDD(D)){continue;}if(D.isTarget&&!D.isLocked()&&D!=S){if(this.isOverTarget(K,D,this.mode,P)){if(H){R.push(D);}else{if(!E[D.id]){O.push(D);}else{B.push(D);}this.dragOvers[D.id]=D;}}}}}this.interactionInfo={out:G,enter:O,over:B,drop:R,point:K,draggedRegion:P,sourceRegion:this.locationCache[S.id],validDrop:H};if(H&&!R.length){this.interactionInfo.validDrop=false;S.onInvalidDrop(Q);}if(this.mode){if(G.length){S.b4DragOut(Q,G);if(S){S.onDragOut(Q,G);}}if(O.length){if(S){S.onDragEnter(Q,O);}}if(B.length){if(S){S.b4DragOver(Q,B);}if(S){S.onDragOver(Q,B);}}if(R.length){if(S){S.b4DragDrop(Q,R);}if(S){S.onDragDrop(Q,R);}}}else{var N=0;for(M=0,N=G.length;M<N;++M){if(S){S.b4DragOut(Q,G[M].id);}if(S){S.onDragOut(Q,G[M].id);}}for(M=0,N=O.length;M<N;++M){if(S){S.onDragEnter(Q,O[M].id);}}for(M=0,N=B.length;M<N;++M){if(S){S.b4DragOver(Q,B[M].id);}if(S){S.onDragOver(Q,B[M].id);}}for(M=0,N=R.length;M<N;++M){if(S){S.b4DragDrop(Q,R[M].id);}if(S){S.onDragDrop(Q,R[M].id);}}}},getBestMatch:function(D){var F=null;var C=D.length;if(C==1){F=D[0];}else{for(var E=0;E<C;++E){var B=D[E];if(this.mode==this.INTERSECT&&B.cursorIsOver){F=B;break;}else{if(!F||!F.overlap||(B.overlap&&F.overlap.getArea()<B.overlap.getArea())){F=B;}}}}return F;},refreshCache:function(C){var E=C||this.ids;for(var B in E){if("string"!=typeof B){continue;}for(var D in this.ids[B]){var F=this.ids[B][D];if(this.isTypeOfDD(F)){var G=this.getLocation(F);if(G){this.locationCache[F.id]=G;}else{delete this.locationCache[F.id];}}}}},verifyEl:function(C){try{if(C){var B=C.offsetParent;if(B){return true;}}}catch(D){}return false;},getLocation:function(G){if(!this.isTypeOfDD(G)){return null;}var E=G.getEl(),J,D,C,L,K,M,B,I,F;try{J=YAHOO.util.Dom.getXY(E);}catch(H){}if(!J){return null;
}D=J[0];C=D+E.offsetWidth;L=J[1];K=L+E.offsetHeight;M=L-G.padding[0];B=C+G.padding[1];I=K+G.padding[2];F=D-G.padding[3];return new YAHOO.util.Region(M,B,I,F);},isOverTarget:function(J,B,D,E){var F=this.locationCache[B.id];if(!F||!this.useCache){F=this.getLocation(B);this.locationCache[B.id]=F;}if(!F){return false;}B.cursorIsOver=F.contains(J);var I=this.dragCurrent;if(!I||(!D&&!I.constrainX&&!I.constrainY)){return B.cursorIsOver;}B.overlap=null;if(!E){var G=I.getTargetCoord(J.x,J.y);var C=I.getDragEl();E=new YAHOO.util.Region(G.y,G.x+C.offsetWidth,G.y+C.offsetHeight,G.x);}var H=E.intersect(F);if(H){B.overlap=H;return(D)?true:B.cursorIsOver;}else{return false;}},_onUnload:function(C,B){this.unregAll();},unregAll:function(){if(this.dragCurrent){this.stopDrag();this.dragCurrent=null;}this._execOnAll("unreg",[]);this.ids={};},elementCache:{},getElWrapper:function(C){var B=this.elementCache[C];if(!B||!B.el){B=this.elementCache[C]=new this.ElementWrapper(YAHOO.util.Dom.get(C));}return B;},getElement:function(B){return YAHOO.util.Dom.get(B);},getCss:function(C){var B=YAHOO.util.Dom.get(C);return(B)?B.style:null;},ElementWrapper:function(B){this.el=B||null;this.id=this.el&&B.id;this.css=this.el&&B.style;},getPosX:function(B){return YAHOO.util.Dom.getX(B);},getPosY:function(B){return YAHOO.util.Dom.getY(B);},swapNode:function(D,B){if(D.swapNode){D.swapNode(B);}else{var E=B.parentNode;var C=B.nextSibling;if(C==D){E.insertBefore(D,B);}else{if(B==D.nextSibling){E.insertBefore(B,D);}else{D.parentNode.replaceChild(B,D);E.insertBefore(D,C);}}}},getScroll:function(){var D,B,E=document.documentElement,C=document.body;if(E&&(E.scrollTop||E.scrollLeft)){D=E.scrollTop;B=E.scrollLeft;}else{if(C){D=C.scrollTop;B=C.scrollLeft;}else{}}return{top:D,left:B};},getStyle:function(C,B){return YAHOO.util.Dom.getStyle(C,B);},getScrollTop:function(){return this.getScroll().top;},getScrollLeft:function(){return this.getScroll().left;},moveToEl:function(B,D){var C=YAHOO.util.Dom.getXY(D);YAHOO.util.Dom.setXY(B,C);},getClientHeight:function(){return YAHOO.util.Dom.getViewportHeight();},getClientWidth:function(){return YAHOO.util.Dom.getViewportWidth();},numericSort:function(C,B){return(C-B);},_timeoutCount:0,_addListeners:function(){var B=YAHOO.util.DDM;if(YAHOO.util.Event&&document){B._onLoad();}else{if(B._timeoutCount>2000){}else{setTimeout(B._addListeners,10);if(document&&document.body){B._timeoutCount+=1;}}}},handleWasClicked:function(B,D){if(this.isHandle(D,B.id)){return true;}else{var C=B.parentNode;while(C){if(this.isHandle(D,C.id)){return true;}else{C=C.parentNode;}}}return false;}};}();YAHOO.util.DDM=YAHOO.util.DragDropMgr;YAHOO.util.DDM._addListeners();}(function(){var A=YAHOO.util.Event;var B=YAHOO.util.Dom;YAHOO.util.DragDrop=function(E,C,D){if(E){this.init(E,C,D);}};YAHOO.util.DragDrop.prototype={id:null,config:null,dragElId:null,handleElId:null,invalidHandleTypes:null,invalidHandleIds:null,invalidHandleClasses:null,startPageX:0,startPageY:0,groups:null,locked:false,lock:function(){this.locked=true;},unlock:function(){this.locked=false;},isTarget:true,padding:null,_domRef:null,__ygDragDrop:true,constrainX:false,constrainY:false,minX:0,maxX:0,minY:0,maxY:0,deltaX:0,deltaY:0,maintainOffset:false,xTicks:null,yTicks:null,primaryButtonOnly:true,available:false,hasOuterHandles:false,cursorIsOver:false,overlap:null,b4StartDrag:function(C,D){},startDrag:function(C,D){},b4Drag:function(C){},onDrag:function(C){},onDragEnter:function(C,D){},b4DragOver:function(C){},onDragOver:function(C,D){},b4DragOut:function(C){},onDragOut:function(C,D){},b4DragDrop:function(C){},onDragDrop:function(C,D){},onInvalidDrop:function(C){},b4EndDrag:function(C){},endDrag:function(C){},b4MouseDown:function(C){},onMouseDown:function(C){},onMouseUp:function(C){},onAvailable:function(){},getEl:function(){if(!this._domRef){this._domRef=B.get(this.id);}return this._domRef;},getDragEl:function(){return B.get(this.dragElId);},init:function(E,C,D){this.initTarget(E,C,D);A.on(this._domRef||this.id,"mousedown",this.handleMouseDown,this,true);},initTarget:function(E,C,D){this.config=D||{};this.DDM=YAHOO.util.DDM;this.groups={};if(typeof E!=="string"){this._domRef=E;E=B.generateId(E);}this.id=E;this.addToGroup((C)?C:"default");this.handleElId=E;A.onAvailable(E,this.handleOnAvailable,this,true);this.setDragElId(E);this.invalidHandleTypes={A:"A"};this.invalidHandleIds={};this.invalidHandleClasses=[];this.applyConfig();},applyConfig:function(){this.padding=this.config.padding||[0,0,0,0];this.isTarget=(this.config.isTarget!==false);this.maintainOffset=(this.config.maintainOffset);this.primaryButtonOnly=(this.config.primaryButtonOnly!==false);},handleOnAvailable:function(){this.available=true;this.resetConstraints();this.onAvailable();},setPadding:function(E,C,F,D){if(!C&&0!==C){this.padding=[E,E,E,E];}else{if(!F&&0!==F){this.padding=[E,C,E,C];}else{this.padding=[E,C,F,D];}}},setInitPosition:function(F,E){var G=this.getEl();if(!this.DDM.verifyEl(G)){return ;}var D=F||0;var C=E||0;var H=B.getXY(G);this.initPageX=H[0]-D;this.initPageY=H[1]-C;this.lastPageX=H[0];this.lastPageY=H[1];this.setStartPosition(H);},setStartPosition:function(D){var C=D||B.getXY(this.getEl());this.deltaSetXY=null;this.startPageX=C[0];this.startPageY=C[1];},addToGroup:function(C){this.groups[C]=true;this.DDM.regDragDrop(this,C);},removeFromGroup:function(C){if(this.groups[C]){delete this.groups[C];}this.DDM.removeDDFromGroup(this,C);},setDragElId:function(C){this.dragElId=C;},setHandleElId:function(C){if(typeof C!=="string"){C=B.generateId(C);}this.handleElId=C;this.DDM.regHandle(this.id,C);},setOuterHandleElId:function(C){if(typeof C!=="string"){C=B.generateId(C);}A.on(C,"mousedown",this.handleMouseDown,this,true);this.setHandleElId(C);this.hasOuterHandles=true;},unreg:function(){A.removeListener(this.id,"mousedown",this.handleMouseDown);this._domRef=null;this.DDM._remove(this);},isLocked:function(){return(this.DDM.isLocked()||this.locked);},handleMouseDown:function(F,E){var C=F.which||F.button;
if(this.primaryButtonOnly&&C>1){return ;}if(this.isLocked()){return ;}this.b4MouseDown(F);this.onMouseDown(F);this.DDM.refreshCache(this.groups);var D=new YAHOO.util.Point(A.getPageX(F),A.getPageY(F));if(!this.hasOuterHandles&&!this.DDM.isOverTarget(D,this)){}else{if(this.clickValidator(F)){this.setStartPosition();this.DDM.handleMouseDown(F,this);this.DDM.stopEvent(F);}else{}}},clickValidator:function(D){var C=A.getTarget(D);return(this.isValidHandleChild(C)&&(this.id==this.handleElId||this.DDM.handleWasClicked(C,this.id)));},getTargetCoord:function(E,D){var C=E-this.deltaX;var F=D-this.deltaY;if(this.constrainX){if(C<this.minX){C=this.minX;}if(C>this.maxX){C=this.maxX;}}if(this.constrainY){if(F<this.minY){F=this.minY;}if(F>this.maxY){F=this.maxY;}}C=this.getTick(C,this.xTicks);F=this.getTick(F,this.yTicks);return{x:C,y:F};},addInvalidHandleType:function(C){var D=C.toUpperCase();this.invalidHandleTypes[D]=D;},addInvalidHandleId:function(C){if(typeof C!=="string"){C=B.generateId(C);}this.invalidHandleIds[C]=C;},addInvalidHandleClass:function(C){this.invalidHandleClasses.push(C);},removeInvalidHandleType:function(C){var D=C.toUpperCase();delete this.invalidHandleTypes[D];},removeInvalidHandleId:function(C){if(typeof C!=="string"){C=B.generateId(C);}delete this.invalidHandleIds[C];},removeInvalidHandleClass:function(D){for(var E=0,C=this.invalidHandleClasses.length;E<C;++E){if(this.invalidHandleClasses[E]==D){delete this.invalidHandleClasses[E];}}},isValidHandleChild:function(F){var E=true;var H;try{H=F.nodeName.toUpperCase();}catch(G){H=F.nodeName;}E=E&&!this.invalidHandleTypes[H];E=E&&!this.invalidHandleIds[F.id];for(var D=0,C=this.invalidHandleClasses.length;E&&D<C;++D){E=!B.hasClass(F,this.invalidHandleClasses[D]);}return E;},setXTicks:function(F,C){this.xTicks=[];this.xTickSize=C;var E={};for(var D=this.initPageX;D>=this.minX;D=D-C){if(!E[D]){this.xTicks[this.xTicks.length]=D;E[D]=true;}}for(D=this.initPageX;D<=this.maxX;D=D+C){if(!E[D]){this.xTicks[this.xTicks.length]=D;E[D]=true;}}this.xTicks.sort(this.DDM.numericSort);},setYTicks:function(F,C){this.yTicks=[];this.yTickSize=C;var E={};for(var D=this.initPageY;D>=this.minY;D=D-C){if(!E[D]){this.yTicks[this.yTicks.length]=D;E[D]=true;}}for(D=this.initPageY;D<=this.maxY;D=D+C){if(!E[D]){this.yTicks[this.yTicks.length]=D;E[D]=true;}}this.yTicks.sort(this.DDM.numericSort);},setXConstraint:function(E,D,C){this.leftConstraint=parseInt(E,10);this.rightConstraint=parseInt(D,10);this.minX=this.initPageX-this.leftConstraint;this.maxX=this.initPageX+this.rightConstraint;if(C){this.setXTicks(this.initPageX,C);}this.constrainX=true;},clearConstraints:function(){this.constrainX=false;this.constrainY=false;this.clearTicks();},clearTicks:function(){this.xTicks=null;this.yTicks=null;this.xTickSize=0;this.yTickSize=0;},setYConstraint:function(C,E,D){this.topConstraint=parseInt(C,10);this.bottomConstraint=parseInt(E,10);this.minY=this.initPageY-this.topConstraint;this.maxY=this.initPageY+this.bottomConstraint;if(D){this.setYTicks(this.initPageY,D);}this.constrainY=true;},resetConstraints:function(){if(this.initPageX||this.initPageX===0){var D=(this.maintainOffset)?this.lastPageX-this.initPageX:0;var C=(this.maintainOffset)?this.lastPageY-this.initPageY:0;this.setInitPosition(D,C);}else{this.setInitPosition();}if(this.constrainX){this.setXConstraint(this.leftConstraint,this.rightConstraint,this.xTickSize);}if(this.constrainY){this.setYConstraint(this.topConstraint,this.bottomConstraint,this.yTickSize);}},getTick:function(I,F){if(!F){return I;}else{if(F[0]>=I){return F[0];}else{for(var D=0,C=F.length;D<C;++D){var E=D+1;if(F[E]&&F[E]>=I){var H=I-F[D];var G=F[E]-I;return(G>H)?F[D]:F[E];}}return F[F.length-1];}}},toString:function(){return("DragDrop "+this.id);}};})();YAHOO.util.DD=function(C,A,B){if(C){this.init(C,A,B);}};YAHOO.extend(YAHOO.util.DD,YAHOO.util.DragDrop,{scroll:true,autoOffset:function(C,B){var A=C-this.startPageX;var D=B-this.startPageY;this.setDelta(A,D);},setDelta:function(B,A){this.deltaX=B;this.deltaY=A;},setDragElPos:function(C,B){var A=this.getDragEl();this.alignElWithMouse(A,C,B);},alignElWithMouse:function(B,F,E){var D=this.getTargetCoord(F,E);if(!this.deltaSetXY){var G=[D.x,D.y];YAHOO.util.Dom.setXY(B,G);var C=parseInt(YAHOO.util.Dom.getStyle(B,"left"),10);var A=parseInt(YAHOO.util.Dom.getStyle(B,"top"),10);this.deltaSetXY=[C-D.x,A-D.y];}else{YAHOO.util.Dom.setStyle(B,"left",(D.x+this.deltaSetXY[0])+"px");YAHOO.util.Dom.setStyle(B,"top",(D.y+this.deltaSetXY[1])+"px");}this.cachePosition(D.x,D.y);this.autoScroll(D.x,D.y,B.offsetHeight,B.offsetWidth);},cachePosition:function(B,A){if(B){this.lastPageX=B;this.lastPageY=A;}else{var C=YAHOO.util.Dom.getXY(this.getEl());this.lastPageX=C[0];this.lastPageY=C[1];}},autoScroll:function(J,I,E,K){if(this.scroll){var L=this.DDM.getClientHeight();var B=this.DDM.getClientWidth();var N=this.DDM.getScrollTop();var D=this.DDM.getScrollLeft();var H=E+I;var M=K+J;var G=(L+N-I-this.deltaY);var F=(B+D-J-this.deltaX);var C=40;var A=(document.all)?80:30;if(H>L&&G<C){window.scrollTo(D,N+A);}if(I<N&&N>0&&I-N<C){window.scrollTo(D,N-A);}if(M>B&&F<C){window.scrollTo(D+A,N);}if(J<D&&D>0&&J-D<C){window.scrollTo(D-A,N);}}},applyConfig:function(){YAHOO.util.DD.superclass.applyConfig.call(this);this.scroll=(this.config.scroll!==false);},b4MouseDown:function(A){this.setStartPosition();this.autoOffset(YAHOO.util.Event.getPageX(A),YAHOO.util.Event.getPageY(A));},b4Drag:function(A){this.setDragElPos(YAHOO.util.Event.getPageX(A),YAHOO.util.Event.getPageY(A));},toString:function(){return("DD "+this.id);}});YAHOO.util.DDProxy=function(C,A,B){if(C){this.init(C,A,B);this.initFrame();}};YAHOO.util.DDProxy.dragElId="ygddfdiv";YAHOO.extend(YAHOO.util.DDProxy,YAHOO.util.DD,{resizeFrame:true,centerFrame:false,createFrame:function(){var B=this,A=document.body;if(!A||!A.firstChild){setTimeout(function(){B.createFrame();},50);return ;}var F=this.getDragEl(),E=YAHOO.util.Dom;if(!F){F=document.createElement("div");F.id=this.dragElId;var D=F.style;
D.position="absolute";D.visibility="hidden";D.cursor="move";D.border="2px solid #aaa";D.zIndex=999;D.height="25px";D.width="25px";var C=document.createElement("div");E.setStyle(C,"height","100%");E.setStyle(C,"width","100%");E.setStyle(C,"background-color","#ccc");E.setStyle(C,"opacity","0");F.appendChild(C);A.insertBefore(F,A.firstChild);}},initFrame:function(){this.createFrame();},applyConfig:function(){YAHOO.util.DDProxy.superclass.applyConfig.call(this);this.resizeFrame=(this.config.resizeFrame!==false);this.centerFrame=(this.config.centerFrame);this.setDragElId(this.config.dragElId||YAHOO.util.DDProxy.dragElId);},showFrame:function(E,D){var C=this.getEl();var A=this.getDragEl();var B=A.style;this._resizeProxy();if(this.centerFrame){this.setDelta(Math.round(parseInt(B.width,10)/2),Math.round(parseInt(B.height,10)/2));}this.setDragElPos(E,D);YAHOO.util.Dom.setStyle(A,"visibility","visible");},_resizeProxy:function(){if(this.resizeFrame){var H=YAHOO.util.Dom;var B=this.getEl();var C=this.getDragEl();var G=parseInt(H.getStyle(C,"borderTopWidth"),10);var I=parseInt(H.getStyle(C,"borderRightWidth"),10);var F=parseInt(H.getStyle(C,"borderBottomWidth"),10);var D=parseInt(H.getStyle(C,"borderLeftWidth"),10);if(isNaN(G)){G=0;}if(isNaN(I)){I=0;}if(isNaN(F)){F=0;}if(isNaN(D)){D=0;}var E=Math.max(0,B.offsetWidth-I-D);var A=Math.max(0,B.offsetHeight-G-F);H.setStyle(C,"width",E+"px");H.setStyle(C,"height",A+"px");}},b4MouseDown:function(B){this.setStartPosition();var A=YAHOO.util.Event.getPageX(B);var C=YAHOO.util.Event.getPageY(B);this.autoOffset(A,C);},b4StartDrag:function(A,B){this.showFrame(A,B);},b4EndDrag:function(A){YAHOO.util.Dom.setStyle(this.getDragEl(),"visibility","hidden");},endDrag:function(D){var C=YAHOO.util.Dom;var B=this.getEl();var A=this.getDragEl();C.setStyle(A,"visibility","");C.setStyle(B,"visibility","hidden");YAHOO.util.DDM.moveToEl(B,A);C.setStyle(A,"visibility","hidden");C.setStyle(B,"visibility","");},toString:function(){return("DDProxy "+this.id);}});YAHOO.util.DDTarget=function(C,A,B){if(C){this.initTarget(C,A,B);}};YAHOO.extend(YAHOO.util.DDTarget,YAHOO.util.DragDrop,{toString:function(){return("DDTarget "+this.id);}});YAHOO.register("dragdrop",YAHOO.util.DragDropMgr,{version:"2.3.1",build:"541"});/*
Copyright (c) 2007, Yahoo! Inc. All rights reserved.
Code licensed under the BSD License:
http://developer.yahoo.net/yui/license.txt
version: 2.3.1
*/
YAHOO.util.Connect={_msxml_progid:["Microsoft.XMLHTTP","MSXML2.XMLHTTP.3.0","MSXML2.XMLHTTP"],_http_headers:{},_has_http_headers:false,_use_default_post_header:true,_default_post_header:"application/x-www-form-urlencoded; charset=UTF-8",_default_form_header:"application/x-www-form-urlencoded",_use_default_xhr_header:true,_default_xhr_header:"XMLHttpRequest",_has_default_headers:true,_default_headers:{},_isFormSubmit:false,_isFileUpload:false,_formNode:null,_sFormData:null,_poll:{},_timeOut:{},_polling_interval:50,_transaction_id:0,_submitElementValue:null,_hasSubmitListener:(function(){if(YAHOO.util.Event){YAHOO.util.Event.addListener(document,"click",function(q){try{var S=YAHOO.util.Event.getTarget(q);if(S.type.toLowerCase()=="submit"){YAHOO.util.Connect._submitElementValue=encodeURIComponent(S.name)+"="+encodeURIComponent(S.value);}}catch(q){}});return true;}return false;})(),startEvent:new YAHOO.util.CustomEvent("start"),completeEvent:new YAHOO.util.CustomEvent("complete"),successEvent:new YAHOO.util.CustomEvent("success"),failureEvent:new YAHOO.util.CustomEvent("failure"),uploadEvent:new YAHOO.util.CustomEvent("upload"),abortEvent:new YAHOO.util.CustomEvent("abort"),_customEvents:{onStart:["startEvent","start"],onComplete:["completeEvent","complete"],onSuccess:["successEvent","success"],onFailure:["failureEvent","failure"],onUpload:["uploadEvent","upload"],onAbort:["abortEvent","abort"]},setProgId:function(S){this._msxml_progid.unshift(S);},setDefaultPostHeader:function(S){if(typeof S=="string"){this._default_post_header=S;}else{if(typeof S=="boolean"){this._use_default_post_header=S;}}},setDefaultXhrHeader:function(S){if(typeof S=="string"){this._default_xhr_header=S;}else{this._use_default_xhr_header=S;}},setPollingInterval:function(S){if(typeof S=="number"&&isFinite(S)){this._polling_interval=S;}},createXhrObject:function(w){var m,S;try{S=new XMLHttpRequest();m={conn:S,tId:w};}catch(R){for(var q=0;q<this._msxml_progid.length;++q){try{S=new ActiveXObject(this._msxml_progid[q]);m={conn:S,tId:w};break;}catch(R){}}}finally{return m;}},getConnectionObject:function(S){var R;var m=this._transaction_id;try{if(!S){R=this.createXhrObject(m);}else{R={};R.tId=m;R.isUpload=true;}if(R){this._transaction_id++;}}catch(q){}finally{return R;}},asyncRequest:function(w,q,m,S){var R=(this._isFileUpload)?this.getConnectionObject(true):this.getConnectionObject();if(!R){return null;}else{if(m&&m.customevents){this.initCustomEvents(R,m);}if(this._isFormSubmit){if(this._isFileUpload){this.uploadFile(R,m,q,S);return R;}if(w.toUpperCase()=="GET"){if(this._sFormData.length!==0){q+=((q.indexOf("?")==-1)?"?":"&")+this._sFormData;}else{q+="?"+this._sFormData;}}else{if(w.toUpperCase()=="POST"){S=S?this._sFormData+"&"+S:this._sFormData;}}}R.conn.open(w,q,true);if(this._use_default_xhr_header){if(!this._default_headers["X-Requested-With"]){this.initHeader("X-Requested-With",this._default_xhr_header,true);}}if(this._isFormSubmit==false&&this._use_default_post_header){this.initHeader("Content-Type",this._default_post_header);}if(this._has_default_headers||this._has_http_headers){this.setHeader(R);}this.handleReadyState(R,m);R.conn.send(S||null);this.startEvent.fire(R);if(R.startEvent){R.startEvent.fire(R);}return R;}},initCustomEvents:function(S,R){for(var q in R.customevents){if(this._customEvents[q][0]){S[this._customEvents[q][0]]=new YAHOO.util.CustomEvent(this._customEvents[q][1],(R.scope)?R.scope:null);S[this._customEvents[q][0]].subscribe(R.customevents[q]);}}},handleReadyState:function(q,R){var S=this;if(R&&R.timeout){this._timeOut[q.tId]=window.setTimeout(function(){S.abort(q,R,true);},R.timeout);}this._poll[q.tId]=window.setInterval(function(){if(q.conn&&q.conn.readyState===4){window.clearInterval(S._poll[q.tId]);delete S._poll[q.tId];if(R&&R.timeout){window.clearTimeout(S._timeOut[q.tId]);delete S._timeOut[q.tId];}S.completeEvent.fire(q);if(q.completeEvent){q.completeEvent.fire(q);}S.handleTransactionResponse(q,R);}},this._polling_interval);},handleTransactionResponse:function(w,V,S){var R,q;try{if(w.conn.status!==undefined&&w.conn.status!==0){R=w.conn.status;}else{R=13030;}}catch(m){R=13030;}if(R>=200&&R<300||R===1223){q=this.createResponseObject(w,(V&&V.argument)?V.argument:undefined);if(V){if(V.success){if(!V.scope){V.success(q);}else{V.success.apply(V.scope,[q]);}}}this.successEvent.fire(q);if(w.successEvent){w.successEvent.fire(q);}}else{switch(R){case 12002:case 12029:case 12030:case 12031:case 12152:case 13030:q=this.createExceptionObject(w.tId,(V&&V.argument)?V.argument:undefined,(S?S:false));if(V){if(V.failure){if(!V.scope){V.failure(q);}else{V.failure.apply(V.scope,[q]);}}}break;default:q=this.createResponseObject(w,(V&&V.argument)?V.argument:undefined);if(V){if(V.failure){if(!V.scope){V.failure(q);}else{V.failure.apply(V.scope,[q]);}}}}this.failureEvent.fire(q);if(w.failureEvent){w.failureEvent.fire(q);}}this.releaseObject(w);q=null;},createResponseObject:function(S,d){var m={};var T={};try{var R=S.conn.getAllResponseHeaders();var V=R.split("\n");for(var w=0;w<V.length;w++){var q=V[w].indexOf(":");if(q!=-1){T[V[w].substring(0,q)]=V[w].substring(q+2);}}}catch(N){}m.tId=S.tId;m.status=(S.conn.status==1223)?204:S.conn.status;m.statusText=(S.conn.status==1223)?"No Content":S.conn.statusText;m.getResponseHeader=T;m.getAllResponseHeaders=R;m.responseText=S.conn.responseText;m.responseXML=S.conn.responseXML;if(typeof d!==undefined){m.argument=d;}return m;},createExceptionObject:function(N,m,S){var V=0;var d="communication failure";var R=-1;var q="transaction aborted";var w={};w.tId=N;if(S){w.status=R;w.statusText=q;}else{w.status=V;w.statusText=d;}if(m){w.argument=m;}return w;},initHeader:function(S,m,R){var q=(R)?this._default_headers:this._http_headers;q[S]=m;if(R){this._has_default_headers=true;}else{this._has_http_headers=true;}},setHeader:function(S){if(this._has_default_headers){for(var q in this._default_headers){if(YAHOO.lang.hasOwnProperty(this._default_headers,q)){S.conn.setRequestHeader(q,this._default_headers[q]);}}}if(this._has_http_headers){for(var q in this._http_headers){if(YAHOO.lang.hasOwnProperty(this._http_headers,q)){S.conn.setRequestHeader(q,this._http_headers[q]);}}delete this._http_headers;this._http_headers={};this._has_http_headers=false;}},resetDefaultHeaders:function(){delete this._default_headers;this._default_headers={};this._has_default_headers=false;},setForm:function(M,w,q){this.resetFormState();var f;if(typeof M=="string"){f=(document.getElementById(M)||document.forms[M]);}else{if(typeof M=="object"){f=M;}else{return ;}}if(w){var V=this.createFrame(q?q:null);this._isFormSubmit=true;this._isFileUpload=true;this._formNode=f;return ;}var S,T,d,p;var N=false;for(var m=0;m<f.elements.length;m++){S=f.elements[m];p=f.elements[m].disabled;T=f.elements[m].name;d=f.elements[m].value;if(!p&&T){switch(S.type){case "select-one":case "select-multiple":for(var R=0;R<S.options.length;R++){if(S.options[R].selected){if(window.ActiveXObject){this._sFormData+=encodeURIComponent(T)+"="+encodeURIComponent(S.options[R].attributes["value"].specified?S.options[R].value:S.options[R].text)+"&";}else{this._sFormData+=encodeURIComponent(T)+"="+encodeURIComponent(S.options[R].hasAttribute("value")?S.options[R].value:S.options[R].text)+"&";}}}break;case "radio":case "checkbox":if(S.checked){this._sFormData+=encodeURIComponent(T)+"="+encodeURIComponent(d)+"&";}break;case "file":case undefined:case "reset":case "button":break;case "submit":if(N===false){if(this._hasSubmitListener&&this._submitElementValue){this._sFormData+=this._submitElementValue+"&";}else{this._sFormData+=encodeURIComponent(T)+"="+encodeURIComponent(d)+"&";}N=true;}break;default:this._sFormData+=encodeURIComponent(T)+"="+encodeURIComponent(d)+"&";}}}this._isFormSubmit=true;this._sFormData=this._sFormData.substr(0,this._sFormData.length-1);this.initHeader("Content-Type",this._default_form_header);return this._sFormData;},resetFormState:function(){this._isFormSubmit=false;this._isFileUpload=false;this._formNode=null;this._sFormData="";},createFrame:function(S){var q="yuiIO"+this._transaction_id;var R;if(window.ActiveXObject){R=document.createElement("<iframe id=\""+q+"\" name=\""+q+"\" />");if(typeof S=="boolean"){R.src="javascript:false";}else{if(typeof secureURI=="string"){R.src=S;}}}else{R=document.createElement("iframe");R.id=q;R.name=q;}R.style.position="absolute";R.style.top="-1000px";R.style.left="-1000px";document.body.appendChild(R);},appendPostData:function(S){var m=[];var q=S.split("&");for(var R=0;R<q.length;R++){var w=q[R].indexOf("=");if(w!=-1){m[R]=document.createElement("input");m[R].type="hidden";m[R].name=q[R].substring(0,w);m[R].value=q[R].substring(w+1);this._formNode.appendChild(m[R]);}}return m;},uploadFile:function(m,p,w,R){var N="yuiIO"+m.tId;var T="multipart/form-data";var f=document.getElementById(N);var U=this;var q={action:this._formNode.getAttribute("action"),method:this._formNode.getAttribute("method"),target:this._formNode.getAttribute("target")};this._formNode.setAttribute("action",w);this._formNode.setAttribute("method","POST");this._formNode.setAttribute("target",N);if(this._formNode.encoding){this._formNode.setAttribute("encoding",T);}else{this._formNode.setAttribute("enctype",T);}if(R){var M=this.appendPostData(R);}this._formNode.submit();this.startEvent.fire(m);if(m.startEvent){m.startEvent.fire(m);}if(p&&p.timeout){this._timeOut[m.tId]=window.setTimeout(function(){U.abort(m,p,true);},p.timeout);}if(M&&M.length>0){for(var d=0;d<M.length;d++){this._formNode.removeChild(M[d]);}}for(var S in q){if(YAHOO.lang.hasOwnProperty(q,S)){if(q[S]){this._formNode.setAttribute(S,q[S]);}else{this._formNode.removeAttribute(S);}}}this.resetFormState();var V=function(){if(p&&p.timeout){window.clearTimeout(U._timeOut[m.tId]);delete U._timeOut[m.tId];}U.completeEvent.fire(m);if(m.completeEvent){m.completeEvent.fire(m);}var v={};v.tId=m.tId;v.argument=p.argument;try{v.responseText=f.contentWindow.document.body?f.contentWindow.document.body.innerHTML:f.contentWindow.document.documentElement.textContent;v.responseXML=f.contentWindow.document.XMLDocument?f.contentWindow.document.XMLDocument:f.contentWindow.document;}catch(u){}if(p&&p.upload){if(!p.scope){p.upload(v);}else{p.upload.apply(p.scope,[v]);}}U.uploadEvent.fire(v);if(m.uploadEvent){m.uploadEvent.fire(v);}YAHOO.util.Event.removeListener(f,"load",V);setTimeout(function(){document.body.removeChild(f);U.releaseObject(m);},100);};YAHOO.util.Event.addListener(f,"load",V);},abort:function(m,V,S){var R;if(m.conn){if(this.isCallInProgress(m)){m.conn.abort();window.clearInterval(this._poll[m.tId]);delete this._poll[m.tId];if(S){window.clearTimeout(this._timeOut[m.tId]);delete this._timeOut[m.tId];}R=true;}}else{if(m.isUpload===true){var q="yuiIO"+m.tId;var w=document.getElementById(q);if(w){YAHOO.util.Event.removeListener(w,"load",uploadCallback);document.body.removeChild(w);if(S){window.clearTimeout(this._timeOut[m.tId]);delete this._timeOut[m.tId];}R=true;}}else{R=false;}}if(R===true){this.abortEvent.fire(m);if(m.abortEvent){m.abortEvent.fire(m);}this.handleTransactionResponse(m,V,true);}return R;},isCallInProgress:function(q){if(q&&q.conn){return q.conn.readyState!==4&&q.conn.readyState!==0;}else{if(q&&q.isUpload===true){var S="yuiIO"+q.tId;return document.getElementById(S)?true:false;}else{return false;}}},releaseObject:function(S){if(S.conn){S.conn=null;}S=null;}};YAHOO.register("connection",YAHOO.util.Connect,{version:"2.3.1",build:"541"});/*
Copyright (c) 2007, Yahoo! Inc. All rights reserved.
Code licensed under the BSD License:
http://developer.yahoo.net/yui/license.txt
version: 2.3.1
*/
YAHOO.widget.AutoComplete=function(G,B,J,D){if(G&&B&&J){if(J instanceof YAHOO.widget.DataSource){this.dataSource=J;}else{return ;}if(YAHOO.util.Dom.inDocument(G)){if(YAHOO.lang.isString(G)){this._sName="instance"+YAHOO.widget.AutoComplete._nIndex+" "+G;this._oTextbox=document.getElementById(G);}else{this._sName=(G.id)?"instance"+YAHOO.widget.AutoComplete._nIndex+" "+G.id:"instance"+YAHOO.widget.AutoComplete._nIndex;this._oTextbox=G;}YAHOO.util.Dom.addClass(this._oTextbox,"yui-ac-input");}else{return ;}if(YAHOO.util.Dom.inDocument(B)){if(YAHOO.lang.isString(B)){this._oContainer=document.getElementById(B);}else{this._oContainer=B;}if(this._oContainer.style.display=="none"){}var E=this._oContainer.parentNode;var A=E.tagName.toLowerCase();if(A=="div"){YAHOO.util.Dom.addClass(E,"yui-ac");}else{}}else{return ;}if(D&&(D.constructor==Object)){for(var I in D){if(I){this[I]=D[I];}}}this._initContainer();this._initProps();this._initList();this._initContainerHelpers();var H=this;var F=this._oTextbox;var C=this._oContainer._oContent;YAHOO.util.Event.addListener(F,"keyup",H._onTextboxKeyUp,H);YAHOO.util.Event.addListener(F,"keydown",H._onTextboxKeyDown,H);YAHOO.util.Event.addListener(F,"focus",H._onTextboxFocus,H);YAHOO.util.Event.addListener(F,"blur",H._onTextboxBlur,H);YAHOO.util.Event.addListener(C,"mouseover",H._onContainerMouseover,H);YAHOO.util.Event.addListener(C,"mouseout",H._onContainerMouseout,H);YAHOO.util.Event.addListener(C,"scroll",H._onContainerScroll,H);YAHOO.util.Event.addListener(C,"resize",H._onContainerResize,H);if(F.form){YAHOO.util.Event.addListener(F.form,"submit",H._onFormSubmit,H);}YAHOO.util.Event.addListener(F,"keypress",H._onTextboxKeyPress,H);this.textboxFocusEvent=new YAHOO.util.CustomEvent("textboxFocus",this);this.textboxKeyEvent=new YAHOO.util.CustomEvent("textboxKey",this);this.dataRequestEvent=new YAHOO.util.CustomEvent("dataRequest",this);this.dataReturnEvent=new YAHOO.util.CustomEvent("dataReturn",this);this.dataErrorEvent=new YAHOO.util.CustomEvent("dataError",this);this.containerExpandEvent=new YAHOO.util.CustomEvent("containerExpand",this);this.typeAheadEvent=new YAHOO.util.CustomEvent("typeAhead",this);this.itemMouseOverEvent=new YAHOO.util.CustomEvent("itemMouseOver",this);this.itemMouseOutEvent=new YAHOO.util.CustomEvent("itemMouseOut",this);this.itemArrowToEvent=new YAHOO.util.CustomEvent("itemArrowTo",this);this.itemArrowFromEvent=new YAHOO.util.CustomEvent("itemArrowFrom",this);this.itemSelectEvent=new YAHOO.util.CustomEvent("itemSelect",this);this.unmatchedItemSelectEvent=new YAHOO.util.CustomEvent("unmatchedItemSelect",this);this.selectionEnforceEvent=new YAHOO.util.CustomEvent("selectionEnforce",this);this.containerCollapseEvent=new YAHOO.util.CustomEvent("containerCollapse",this);this.textboxBlurEvent=new YAHOO.util.CustomEvent("textboxBlur",this);F.setAttribute("autocomplete","off");YAHOO.widget.AutoComplete._nIndex++;}else{}};YAHOO.widget.AutoComplete.prototype.dataSource=null;YAHOO.widget.AutoComplete.prototype.minQueryLength=1;YAHOO.widget.AutoComplete.prototype.maxResultsDisplayed=10;YAHOO.widget.AutoComplete.prototype.queryDelay=0.2;YAHOO.widget.AutoComplete.prototype.highlightClassName="yui-ac-highlight";YAHOO.widget.AutoComplete.prototype.prehighlightClassName=null;YAHOO.widget.AutoComplete.prototype.delimChar=null;YAHOO.widget.AutoComplete.prototype.autoHighlight=true;YAHOO.widget.AutoComplete.prototype.typeAhead=false;YAHOO.widget.AutoComplete.prototype.animHoriz=false;YAHOO.widget.AutoComplete.prototype.animVert=true;YAHOO.widget.AutoComplete.prototype.animSpeed=0.3;YAHOO.widget.AutoComplete.prototype.forceSelection=false;YAHOO.widget.AutoComplete.prototype.allowBrowserAutocomplete=true;YAHOO.widget.AutoComplete.prototype.alwaysShowContainer=false;YAHOO.widget.AutoComplete.prototype.useIFrame=false;YAHOO.widget.AutoComplete.prototype.useShadow=false;YAHOO.widget.AutoComplete.prototype.toString=function(){return"AutoComplete "+this._sName;};YAHOO.widget.AutoComplete.prototype.isContainerOpen=function(){return this._bContainerOpen;};YAHOO.widget.AutoComplete.prototype.getListItems=function(){return this._aListItems;};YAHOO.widget.AutoComplete.prototype.getListItemData=function(A){if(A._oResultData){return A._oResultData;}else{return false;}};YAHOO.widget.AutoComplete.prototype.setHeader=function(A){if(A){if(this._oContainer._oContent._oHeader){this._oContainer._oContent._oHeader.innerHTML=A;this._oContainer._oContent._oHeader.style.display="block";}}else{this._oContainer._oContent._oHeader.innerHTML="";this._oContainer._oContent._oHeader.style.display="none";}};YAHOO.widget.AutoComplete.prototype.setFooter=function(A){if(A){if(this._oContainer._oContent._oFooter){this._oContainer._oContent._oFooter.innerHTML=A;this._oContainer._oContent._oFooter.style.display="block";}}else{this._oContainer._oContent._oFooter.innerHTML="";this._oContainer._oContent._oFooter.style.display="none";}};YAHOO.widget.AutoComplete.prototype.setBody=function(A){if(A){if(this._oContainer._oContent._oBody){this._oContainer._oContent._oBody.innerHTML=A;this._oContainer._oContent._oBody.style.display="block";this._oContainer._oContent.style.display="block";}}else{this._oContainer._oContent._oBody.innerHTML="";this._oContainer._oContent.style.display="none";}this._maxResultsDisplayed=0;};YAHOO.widget.AutoComplete.prototype.formatResult=function(B,C){var A=B[0];if(A){return A;}else{return"";}};YAHOO.widget.AutoComplete.prototype.doBeforeExpandContainer=function(A,B,D,C){return true;};YAHOO.widget.AutoComplete.prototype.sendQuery=function(A){this._sendQuery(A);};YAHOO.widget.AutoComplete.prototype.doBeforeSendQuery=function(A){return A;};YAHOO.widget.AutoComplete.prototype.destroy=function(){var B=this.toString();var A=this._oTextbox;var D=this._oContainer;this.textboxFocusEvent.unsubscribe();this.textboxKeyEvent.unsubscribe();this.dataRequestEvent.unsubscribe();this.dataReturnEvent.unsubscribe();this.dataErrorEvent.unsubscribe();this.containerExpandEvent.unsubscribe();this.typeAheadEvent.unsubscribe();
this.itemMouseOverEvent.unsubscribe();this.itemMouseOutEvent.unsubscribe();this.itemArrowToEvent.unsubscribe();this.itemArrowFromEvent.unsubscribe();this.itemSelectEvent.unsubscribe();this.unmatchedItemSelectEvent.unsubscribe();this.selectionEnforceEvent.unsubscribe();this.containerCollapseEvent.unsubscribe();this.textboxBlurEvent.unsubscribe();YAHOO.util.Event.purgeElement(A,true);YAHOO.util.Event.purgeElement(D,true);D.innerHTML="";for(var C in this){if(YAHOO.lang.hasOwnProperty(this,C)){this[C]=null;}}};YAHOO.widget.AutoComplete.prototype.textboxFocusEvent=null;YAHOO.widget.AutoComplete.prototype.textboxKeyEvent=null;YAHOO.widget.AutoComplete.prototype.dataRequestEvent=null;YAHOO.widget.AutoComplete.prototype.dataReturnEvent=null;YAHOO.widget.AutoComplete.prototype.dataErrorEvent=null;YAHOO.widget.AutoComplete.prototype.containerExpandEvent=null;YAHOO.widget.AutoComplete.prototype.typeAheadEvent=null;YAHOO.widget.AutoComplete.prototype.itemMouseOverEvent=null;YAHOO.widget.AutoComplete.prototype.itemMouseOutEvent=null;YAHOO.widget.AutoComplete.prototype.itemArrowToEvent=null;YAHOO.widget.AutoComplete.prototype.itemArrowFromEvent=null;YAHOO.widget.AutoComplete.prototype.itemSelectEvent=null;YAHOO.widget.AutoComplete.prototype.unmatchedItemSelectEvent=null;YAHOO.widget.AutoComplete.prototype.selectionEnforceEvent=null;YAHOO.widget.AutoComplete.prototype.containerCollapseEvent=null;YAHOO.widget.AutoComplete.prototype.textboxBlurEvent=null;YAHOO.widget.AutoComplete._nIndex=0;YAHOO.widget.AutoComplete.prototype._sName=null;YAHOO.widget.AutoComplete.prototype._oTextbox=null;YAHOO.widget.AutoComplete.prototype._bFocused=true;YAHOO.widget.AutoComplete.prototype._oAnim=null;YAHOO.widget.AutoComplete.prototype._oContainer=null;YAHOO.widget.AutoComplete.prototype._bContainerOpen=false;YAHOO.widget.AutoComplete.prototype._bOverContainer=false;YAHOO.widget.AutoComplete.prototype._aListItems=null;YAHOO.widget.AutoComplete.prototype._nDisplayedItems=0;YAHOO.widget.AutoComplete.prototype._maxResultsDisplayed=0;YAHOO.widget.AutoComplete.prototype._sCurQuery=null;YAHOO.widget.AutoComplete.prototype._sSavedQuery=null;YAHOO.widget.AutoComplete.prototype._oCurItem=null;YAHOO.widget.AutoComplete.prototype._bItemSelected=false;YAHOO.widget.AutoComplete.prototype._nKeyCode=null;YAHOO.widget.AutoComplete.prototype._nDelayID=-1;YAHOO.widget.AutoComplete.prototype._iFrameSrc="javascript:false;";YAHOO.widget.AutoComplete.prototype._queryInterval=null;YAHOO.widget.AutoComplete.prototype._sLastTextboxValue=null;YAHOO.widget.AutoComplete.prototype._initProps=function(){var B=this.minQueryLength;if(!YAHOO.lang.isNumber(B)){this.minQueryLength=1;}var D=this.maxResultsDisplayed;if(!YAHOO.lang.isNumber(D)||(D<1)){this.maxResultsDisplayed=10;}var E=this.queryDelay;if(!YAHOO.lang.isNumber(E)||(E<0)){this.queryDelay=0.2;}var A=this.delimChar;if(YAHOO.lang.isString(A)&&(A.length>0)){this.delimChar=[A];}else{if(!YAHOO.lang.isArray(A)){this.delimChar=null;}}var C=this.animSpeed;if((this.animHoriz||this.animVert)&&YAHOO.util.Anim){if(!YAHOO.lang.isNumber(C)||(C<0)){this.animSpeed=0.3;}if(!this._oAnim){this._oAnim=new YAHOO.util.Anim(this._oContainer._oContent,{},this.animSpeed);}else{this._oAnim.duration=this.animSpeed;}}if(this.forceSelection&&A){}};YAHOO.widget.AutoComplete.prototype._initContainerHelpers=function(){if(this.useShadow&&!this._oContainer._oShadow){var B=document.createElement("div");B.className="yui-ac-shadow";this._oContainer._oShadow=this._oContainer.appendChild(B);}if(this.useIFrame&&!this._oContainer._oIFrame){var A=document.createElement("iframe");A.src=this._iFrameSrc;A.frameBorder=0;A.scrolling="no";A.style.position="absolute";A.style.width="100%";A.style.height="100%";A.tabIndex=-1;this._oContainer._oIFrame=this._oContainer.appendChild(A);}};YAHOO.widget.AutoComplete.prototype._initContainer=function(){YAHOO.util.Dom.addClass(this._oContainer,"yui-ac-container");if(!this._oContainer._oContent){var D=document.createElement("div");D.className="yui-ac-content";D.style.display="none";this._oContainer._oContent=this._oContainer.appendChild(D);var B=document.createElement("div");B.className="yui-ac-hd";B.style.display="none";this._oContainer._oContent._oHeader=this._oContainer._oContent.appendChild(B);var C=document.createElement("div");C.className="yui-ac-bd";this._oContainer._oContent._oBody=this._oContainer._oContent.appendChild(C);var A=document.createElement("div");A.className="yui-ac-ft";A.style.display="none";this._oContainer._oContent._oFooter=this._oContainer._oContent.appendChild(A);}else{}};YAHOO.widget.AutoComplete.prototype._initList=function(){this._aListItems=[];while(this._oContainer._oContent._oBody.hasChildNodes()){var B=this.getListItems();if(B){for(var A=B.length-1;A>=0;A--){B[A]=null;}}this._oContainer._oContent._oBody.innerHTML="";}var E=document.createElement("ul");E=this._oContainer._oContent._oBody.appendChild(E);for(var C=0;C<this.maxResultsDisplayed;C++){var D=document.createElement("li");D=E.appendChild(D);this._aListItems[C]=D;this._initListItem(D,C);}this._maxResultsDisplayed=this.maxResultsDisplayed;};YAHOO.widget.AutoComplete.prototype._initListItem=function(C,B){var A=this;C.style.display="none";C._nItemIndex=B;C.mouseover=C.mouseout=C.onclick=null;YAHOO.util.Event.addListener(C,"mouseover",A._onItemMouseover,A);YAHOO.util.Event.addListener(C,"mouseout",A._onItemMouseout,A);YAHOO.util.Event.addListener(C,"click",A._onItemMouseclick,A);};YAHOO.widget.AutoComplete.prototype._onIMEDetected=function(A){A._enableIntervalDetection();};YAHOO.widget.AutoComplete.prototype._enableIntervalDetection=function(){var A=this._oTextbox.value;var B=this._sLastTextboxValue;if(A!=B){this._sLastTextboxValue=A;this._sendQuery(A);}};YAHOO.widget.AutoComplete.prototype._cancelIntervalDetection=function(A){if(A._queryInterval){clearInterval(A._queryInterval);}};YAHOO.widget.AutoComplete.prototype._isIgnoreKey=function(A){if((A==9)||(A==13)||(A==16)||(A==17)||(A>=18&&A<=20)||(A==27)||(A>=33&&A<=35)||(A>=36&&A<=40)||(A>=44&&A<=45)){return true;
}return false;};YAHOO.widget.AutoComplete.prototype._sendQuery=function(G){if(this.minQueryLength==-1){this._toggleContainer(false);return ;}var C=(this.delimChar)?this.delimChar:null;if(C){var E=-1;for(var B=C.length-1;B>=0;B--){var F=G.lastIndexOf(C[B]);if(F>E){E=F;}}if(C[B]==" "){for(var A=C.length-1;A>=0;A--){if(G[E-1]==C[A]){E--;break;}}}if(E>-1){var D=E+1;while(G.charAt(D)==" "){D+=1;}this._sSavedQuery=G.substring(0,D);G=G.substr(D);}else{if(G.indexOf(this._sSavedQuery)<0){this._sSavedQuery=null;}}}if((G&&(G.length<this.minQueryLength))||(!G&&this.minQueryLength>0)){if(this._nDelayID!=-1){clearTimeout(this._nDelayID);}this._toggleContainer(false);return ;}G=encodeURIComponent(G);this._nDelayID=-1;G=this.doBeforeSendQuery(G);this.dataRequestEvent.fire(this,G);this.dataSource.getResults(this._populateList,G,this);};YAHOO.widget.AutoComplete.prototype._populateList=function(K,L,I){if(L===null){I.dataErrorEvent.fire(I,K);}if(!I._bFocused||!L){return ;}var A=(navigator.userAgent.toLowerCase().indexOf("opera")!=-1);var O=I._oContainer._oContent.style;O.width=(!A)?null:"";O.height=(!A)?null:"";var H=decodeURIComponent(K);I._sCurQuery=H;I._bItemSelected=false;if(I._maxResultsDisplayed!=I.maxResultsDisplayed){I._initList();}var C=Math.min(L.length,I.maxResultsDisplayed);I._nDisplayedItems=C;if(C>0){I._initContainerHelpers();var D=I._aListItems;for(var G=C-1;G>=0;G--){var N=D[G];var B=L[G];N.innerHTML=I.formatResult(B,H);N.style.display="list-item";N._sResultKey=B[0];N._oResultData=B;}for(var F=D.length-1;F>=C;F--){var M=D[F];M.innerHTML=null;M.style.display="none";M._sResultKey=null;M._oResultData=null;}var J=I.doBeforeExpandContainer(I._oTextbox,I._oContainer,K,L);I._toggleContainer(J);if(I.autoHighlight){var E=D[0];I._toggleHighlight(E,"to");I.itemArrowToEvent.fire(I,E);I._typeAhead(E,K);}else{I._oCurItem=null;}}else{I._toggleContainer(false);}I.dataReturnEvent.fire(I,K,L);};YAHOO.widget.AutoComplete.prototype._clearSelection=function(){var C=this._oTextbox.value;var B=(this.delimChar)?this.delimChar[0]:null;var A=(B)?C.lastIndexOf(B,C.length-2):-1;if(A>-1){this._oTextbox.value=C.substring(0,A);}else{this._oTextbox.value="";}this._sSavedQuery=this._oTextbox.value;this.selectionEnforceEvent.fire(this);};YAHOO.widget.AutoComplete.prototype._textMatchesOption=function(){var D=null;for(var A=this._nDisplayedItems-1;A>=0;A--){var C=this._aListItems[A];var B=C._sResultKey.toLowerCase();if(B==this._sCurQuery.toLowerCase()){D=C;break;}}return(D);};YAHOO.widget.AutoComplete.prototype._typeAhead=function(E,G){if(!this.typeAhead||(this._nKeyCode==8)){return ;}var B=this._oTextbox;var F=this._oTextbox.value;if(!B.setSelectionRange&&!B.createTextRange){return ;}var C=F.length;this._updateValue(E);var D=B.value.length;this._selectText(B,C,D);var A=B.value.substr(C,D);this.typeAheadEvent.fire(this,G,A);};YAHOO.widget.AutoComplete.prototype._selectText=function(A,B,C){if(A.setSelectionRange){A.setSelectionRange(B,C);}else{if(A.createTextRange){var D=A.createTextRange();D.moveStart("character",B);D.moveEnd("character",C-A.value.length);D.select();}else{A.select();}}};YAHOO.widget.AutoComplete.prototype._toggleContainerHelpers=function(B){var D=false;var C=this._oContainer._oContent.offsetWidth+"px";var A=this._oContainer._oContent.offsetHeight+"px";if(this.useIFrame&&this._oContainer._oIFrame){D=true;if(B){this._oContainer._oIFrame.style.width=C;this._oContainer._oIFrame.style.height=A;}else{this._oContainer._oIFrame.style.width=0;this._oContainer._oIFrame.style.height=0;}}if(this.useShadow&&this._oContainer._oShadow){D=true;if(B){this._oContainer._oShadow.style.width=C;this._oContainer._oShadow.style.height=A;}else{this._oContainer._oShadow.style.width=0;this._oContainer._oShadow.style.height=0;}}};YAHOO.widget.AutoComplete.prototype._toggleContainer=function(J){var L=this._oContainer;if(this.alwaysShowContainer&&this._bContainerOpen){return ;}if(!J){this._oContainer._oContent.scrollTop=0;var C=this._aListItems;if(C&&(C.length>0)){for(var G=C.length-1;G>=0;G--){C[G].style.display="none";}}if(this._oCurItem){this._toggleHighlight(this._oCurItem,"from");}this._oCurItem=null;this._nDisplayedItems=0;this._sCurQuery=null;}if(!J&&!this._bContainerOpen){L._oContent.style.display="none";return ;}var B=this._oAnim;if(B&&B.getEl()&&(this.animHoriz||this.animVert)){if(!J){this._toggleContainerHelpers(J);}if(B.isAnimated()){B.stop();}var H=L._oContent.cloneNode(true);L.appendChild(H);H.style.top="-9000px";H.style.display="block";var F=H.offsetWidth;var D=H.offsetHeight;var A=(this.animHoriz)?0:F;var E=(this.animVert)?0:D;B.attributes=(J)?{width:{to:F},height:{to:D}}:{width:{to:A},height:{to:E}};if(J&&!this._bContainerOpen){L._oContent.style.width=A+"px";L._oContent.style.height=E+"px";}else{L._oContent.style.width=F+"px";L._oContent.style.height=D+"px";}L.removeChild(H);H=null;var I=this;var K=function(){B.onComplete.unsubscribeAll();if(J){I.containerExpandEvent.fire(I);}else{L._oContent.style.display="none";I.containerCollapseEvent.fire(I);}I._toggleContainerHelpers(J);};L._oContent.style.display="block";B.onComplete.subscribe(K);B.animate();this._bContainerOpen=J;}else{if(J){L._oContent.style.display="block";this.containerExpandEvent.fire(this);}else{L._oContent.style.display="none";this.containerCollapseEvent.fire(this);}this._toggleContainerHelpers(J);this._bContainerOpen=J;}};YAHOO.widget.AutoComplete.prototype._toggleHighlight=function(A,C){var B=this.highlightClassName;if(this._oCurItem){YAHOO.util.Dom.removeClass(this._oCurItem,B);}if((C=="to")&&B){YAHOO.util.Dom.addClass(A,B);this._oCurItem=A;}};YAHOO.widget.AutoComplete.prototype._togglePrehighlight=function(A,C){if(A==this._oCurItem){return ;}var B=this.prehighlightClassName;if((C=="mouseover")&&B){YAHOO.util.Dom.addClass(A,B);}else{YAHOO.util.Dom.removeClass(A,B);}};YAHOO.widget.AutoComplete.prototype._updateValue=function(F){var C=this._oTextbox;var E=(this.delimChar)?(this.delimChar[0]||this.delimChar):null;var B=this._sSavedQuery;var D=F._sResultKey;C.focus();
C.value="";if(E){if(B){C.value=B;}C.value+=D+E;if(E!=" "){C.value+=" ";}}else{C.value=D;}if(C.type=="textarea"){C.scrollTop=C.scrollHeight;}var A=C.value.length;this._selectText(C,A,A);this._oCurItem=F;};YAHOO.widget.AutoComplete.prototype._selectItem=function(A){this._bItemSelected=true;this._updateValue(A);this._cancelIntervalDetection(this);this.itemSelectEvent.fire(this,A,A._oResultData);this._toggleContainer(false);};YAHOO.widget.AutoComplete.prototype._jumpSelection=function(){if(this._oCurItem){this._selectItem(this._oCurItem);}else{this._toggleContainer(false);}};YAHOO.widget.AutoComplete.prototype._moveSelection=function(G){if(this._bContainerOpen){var D=this._oCurItem;var F=-1;if(D){F=D._nItemIndex;}var C=(G==40)?(F+1):(F-1);if(C<-2||C>=this._nDisplayedItems){return ;}if(D){this._toggleHighlight(D,"from");this.itemArrowFromEvent.fire(this,D);}if(C==-1){if(this.delimChar&&this._sSavedQuery){if(!this._textMatchesOption()){this._oTextbox.value=this._sSavedQuery;}else{this._oTextbox.value=this._sSavedQuery+this._sCurQuery;}}else{this._oTextbox.value=this._sCurQuery;}this._oCurItem=null;return ;}if(C==-2){this._toggleContainer(false);return ;}var B=this._aListItems[C];var E=this._oContainer._oContent;var A=((YAHOO.util.Dom.getStyle(E,"overflow")=="auto")||(YAHOO.util.Dom.getStyle(E,"overflowY")=="auto"));if(A&&(C>-1)&&(C<this._nDisplayedItems)){if(G==40){if((B.offsetTop+B.offsetHeight)>(E.scrollTop+E.offsetHeight)){E.scrollTop=(B.offsetTop+B.offsetHeight)-E.offsetHeight;}else{if((B.offsetTop+B.offsetHeight)<E.scrollTop){E.scrollTop=B.offsetTop;}}}else{if(B.offsetTop<E.scrollTop){this._oContainer._oContent.scrollTop=B.offsetTop;}else{if(B.offsetTop>(E.scrollTop+E.offsetHeight)){this._oContainer._oContent.scrollTop=(B.offsetTop+B.offsetHeight)-E.offsetHeight;}}}}this._toggleHighlight(B,"to");this.itemArrowToEvent.fire(this,B);if(this.typeAhead){this._updateValue(B);}}};YAHOO.widget.AutoComplete.prototype._onItemMouseover=function(A,B){if(B.prehighlightClassName){B._togglePrehighlight(this,"mouseover");}else{B._toggleHighlight(this,"to");}B.itemMouseOverEvent.fire(B,this);};YAHOO.widget.AutoComplete.prototype._onItemMouseout=function(A,B){if(B.prehighlightClassName){B._togglePrehighlight(this,"mouseout");}else{B._toggleHighlight(this,"from");}B.itemMouseOutEvent.fire(B,this);};YAHOO.widget.AutoComplete.prototype._onItemMouseclick=function(A,B){B._toggleHighlight(this,"to");B._selectItem(this);};YAHOO.widget.AutoComplete.prototype._onContainerMouseover=function(A,B){B._bOverContainer=true;};YAHOO.widget.AutoComplete.prototype._onContainerMouseout=function(A,B){B._bOverContainer=false;if(B._oCurItem){B._toggleHighlight(B._oCurItem,"to");}};YAHOO.widget.AutoComplete.prototype._onContainerScroll=function(A,B){B._oTextbox.focus();};YAHOO.widget.AutoComplete.prototype._onContainerResize=function(A,B){B._toggleContainerHelpers(B._bContainerOpen);};YAHOO.widget.AutoComplete.prototype._onTextboxKeyDown=function(A,B){var C=A.keyCode;switch(C){case 9:if(B._oCurItem){if(B.delimChar&&(B._nKeyCode!=C)){if(B._bContainerOpen){YAHOO.util.Event.stopEvent(A);}}B._selectItem(B._oCurItem);}else{B._toggleContainer(false);}break;case 13:if(B._oCurItem){if(B._nKeyCode!=C){if(B._bContainerOpen){YAHOO.util.Event.stopEvent(A);}}B._selectItem(B._oCurItem);}else{B._toggleContainer(false);}break;case 27:B._toggleContainer(false);return ;case 39:B._jumpSelection();break;case 38:YAHOO.util.Event.stopEvent(A);B._moveSelection(C);break;case 40:YAHOO.util.Event.stopEvent(A);B._moveSelection(C);break;default:break;}};YAHOO.widget.AutoComplete.prototype._onTextboxKeyPress=function(A,C){var D=A.keyCode;var B=(navigator.userAgent.toLowerCase().indexOf("mac")!=-1);if(B){switch(D){case 9:if(C._oCurItem){if(C.delimChar&&(C._nKeyCode!=D)){YAHOO.util.Event.stopEvent(A);}}break;case 13:if(C._oCurItem){if(C._nKeyCode!=D){if(C._bContainerOpen){YAHOO.util.Event.stopEvent(A);}}}break;case 38:case 40:YAHOO.util.Event.stopEvent(A);break;default:break;}}else{if(D==229){C._queryInterval=setInterval(function(){C._onIMEDetected(C);},500);}}};YAHOO.widget.AutoComplete.prototype._onTextboxKeyUp=function(B,D){D._initProps();var E=B.keyCode;D._nKeyCode=E;var C=this.value;if(D._isIgnoreKey(E)||(C.toLowerCase()==D._sCurQuery)){return ;}else{D._bItemSelected=false;YAHOO.util.Dom.removeClass(D._oCurItem,D.highlightClassName);D._oCurItem=null;D.textboxKeyEvent.fire(D,E);}if(D.queryDelay>0){var A=setTimeout(function(){D._sendQuery(C);},(D.queryDelay*1000));if(D._nDelayID!=-1){clearTimeout(D._nDelayID);}D._nDelayID=A;}else{D._sendQuery(C);}};YAHOO.widget.AutoComplete.prototype._onTextboxFocus=function(A,B){B._oTextbox.setAttribute("autocomplete","off");B._bFocused=true;if(!B._bItemSelected){B.textboxFocusEvent.fire(B);}};YAHOO.widget.AutoComplete.prototype._onTextboxBlur=function(A,B){if(!B._bOverContainer||(B._nKeyCode==9)){if(!B._bItemSelected){var C=B._textMatchesOption();if(!B._bContainerOpen||(B._bContainerOpen&&(C===null))){if(B.forceSelection){B._clearSelection();}else{B.unmatchedItemSelectEvent.fire(B);}}else{if(B.forceSelection){B._selectItem(C);}}}if(B._bContainerOpen){B._toggleContainer(false);}B._cancelIntervalDetection(B);B._bFocused=false;B.textboxBlurEvent.fire(B);}};YAHOO.widget.AutoComplete.prototype._onFormSubmit=function(A,B){if(B.allowBrowserAutocomplete){B._oTextbox.setAttribute("autocomplete","on");}else{B._oTextbox.setAttribute("autocomplete","off");}};YAHOO.widget.DataSource=function(){};YAHOO.widget.DataSource.ERROR_DATANULL="Response data was null";YAHOO.widget.DataSource.ERROR_DATAPARSE="Response data could not be parsed";YAHOO.widget.DataSource.prototype.maxCacheEntries=15;YAHOO.widget.DataSource.prototype.queryMatchContains=false;YAHOO.widget.DataSource.prototype.queryMatchSubset=false;YAHOO.widget.DataSource.prototype.queryMatchCase=false;YAHOO.widget.DataSource.prototype.toString=function(){return"DataSource "+this._sName;};YAHOO.widget.DataSource.prototype.getResults=function(A,D,B){var C=this._doQueryCache(A,D,B);if(C.length===0){this.queryEvent.fire(this,B,D);
this.doQuery(A,D,B);}};YAHOO.widget.DataSource.prototype.doQuery=function(A,C,B){};YAHOO.widget.DataSource.prototype.flushCache=function(){if(this._aCache){this._aCache=[];}if(this._aCacheHelper){this._aCacheHelper=[];}this.cacheFlushEvent.fire(this);};YAHOO.widget.DataSource.prototype.queryEvent=null;YAHOO.widget.DataSource.prototype.cacheQueryEvent=null;YAHOO.widget.DataSource.prototype.getResultsEvent=null;YAHOO.widget.DataSource.prototype.getCachedResultsEvent=null;YAHOO.widget.DataSource.prototype.dataErrorEvent=null;YAHOO.widget.DataSource.prototype.cacheFlushEvent=null;YAHOO.widget.DataSource._nIndex=0;YAHOO.widget.DataSource.prototype._sName=null;YAHOO.widget.DataSource.prototype._aCache=null;YAHOO.widget.DataSource.prototype._init=function(){var A=this.maxCacheEntries;if(!YAHOO.lang.isNumber(A)||(A<0)){A=0;}if(A>0&&!this._aCache){this._aCache=[];}this._sName="instance"+YAHOO.widget.DataSource._nIndex;YAHOO.widget.DataSource._nIndex++;this.queryEvent=new YAHOO.util.CustomEvent("query",this);this.cacheQueryEvent=new YAHOO.util.CustomEvent("cacheQuery",this);this.getResultsEvent=new YAHOO.util.CustomEvent("getResults",this);this.getCachedResultsEvent=new YAHOO.util.CustomEvent("getCachedResults",this);this.dataErrorEvent=new YAHOO.util.CustomEvent("dataError",this);this.cacheFlushEvent=new YAHOO.util.CustomEvent("cacheFlush",this);};YAHOO.widget.DataSource.prototype._addCacheElem=function(B){var A=this._aCache;if(!A||!B||!B.query||!B.results){return ;}if(A.length>=this.maxCacheEntries){A.shift();}A.push(B);};YAHOO.widget.DataSource.prototype._doQueryCache=function(A,I,N){var H=[];var G=false;var J=this._aCache;var F=(J)?J.length:0;var K=this.queryMatchContains;var D;if((this.maxCacheEntries>0)&&J&&(F>0)){this.cacheQueryEvent.fire(this,N,I);if(!this.queryMatchCase){D=I;I=I.toLowerCase();}for(var P=F-1;P>=0;P--){var E=J[P];var B=E.results;var C=(!this.queryMatchCase)?encodeURIComponent(E.query).toLowerCase():encodeURIComponent(E.query);if(C==I){G=true;H=B;if(P!=F-1){J.splice(P,1);this._addCacheElem(E);}break;}else{if(this.queryMatchSubset){for(var O=I.length-1;O>=0;O--){var R=I.substr(0,O);if(C==R){G=true;for(var M=B.length-1;M>=0;M--){var Q=B[M];var L=(this.queryMatchCase)?encodeURIComponent(Q[0]).indexOf(I):encodeURIComponent(Q[0]).toLowerCase().indexOf(I);if((!K&&(L===0))||(K&&(L>-1))){H.unshift(Q);}}E={};E.query=I;E.results=H;this._addCacheElem(E);break;}}if(G){break;}}}}if(G){this.getCachedResultsEvent.fire(this,N,D,H);A(D,H,N);}}return H;};YAHOO.widget.DS_XHR=function(C,A,D){if(D&&(D.constructor==Object)){for(var B in D){this[B]=D[B];}}if(!YAHOO.lang.isArray(A)||!YAHOO.lang.isString(C)){return ;}this.schema=A;this.scriptURI=C;this._init();};YAHOO.widget.DS_XHR.prototype=new YAHOO.widget.DataSource();YAHOO.widget.DS_XHR.TYPE_JSON=0;YAHOO.widget.DS_XHR.TYPE_XML=1;YAHOO.widget.DS_XHR.TYPE_FLAT=2;YAHOO.widget.DS_XHR.ERROR_DATAXHR="XHR response failed";YAHOO.widget.DS_XHR.prototype.connMgr=YAHOO.util.Connect;YAHOO.widget.DS_XHR.prototype.connTimeout=0;YAHOO.widget.DS_XHR.prototype.scriptURI=null;YAHOO.widget.DS_XHR.prototype.scriptQueryParam="query";YAHOO.widget.DS_XHR.prototype.scriptQueryAppend="";YAHOO.widget.DS_XHR.prototype.responseType=YAHOO.widget.DS_XHR.TYPE_JSON;YAHOO.widget.DS_XHR.prototype.responseStripAfter="\n<!-";YAHOO.widget.DS_XHR.prototype.doQuery=function(E,G,B){var J=(this.responseType==YAHOO.widget.DS_XHR.TYPE_XML);var D=this.scriptURI+"?"+this.scriptQueryParam+"="+G;if(this.scriptQueryAppend.length>0){D+="&"+this.scriptQueryAppend;}var C=null;var F=this;var I=function(K){if(!F._oConn||(K.tId!=F._oConn.tId)){F.dataErrorEvent.fire(F,B,G,YAHOO.widget.DataSource.ERROR_DATANULL);return ;}for(var N in K){}if(!J){K=K.responseText;}else{K=K.responseXML;}if(K===null){F.dataErrorEvent.fire(F,B,G,YAHOO.widget.DataSource.ERROR_DATANULL);return ;}var M=F.parseResponse(G,K,B);var L={};L.query=decodeURIComponent(G);L.results=M;if(M===null){F.dataErrorEvent.fire(F,B,G,YAHOO.widget.DataSource.ERROR_DATAPARSE);M=[];}else{F.getResultsEvent.fire(F,B,G,M);F._addCacheElem(L);}E(G,M,B);};var A=function(K){F.dataErrorEvent.fire(F,B,G,YAHOO.widget.DS_XHR.ERROR_DATAXHR);return ;};var H={success:I,failure:A};if(YAHOO.lang.isNumber(this.connTimeout)&&(this.connTimeout>0)){H.timeout=this.connTimeout;}if(this._oConn){this.connMgr.abort(this._oConn);}F._oConn=this.connMgr.asyncRequest("GET",D,H,null);};YAHOO.widget.DS_XHR.prototype.parseResponse=function(sQuery,oResponse,oParent){var aSchema=this.schema;var aResults=[];var bError=false;var nEnd=((this.responseStripAfter!=="")&&(oResponse.indexOf))?oResponse.indexOf(this.responseStripAfter):-1;if(nEnd!=-1){oResponse=oResponse.substring(0,nEnd);}switch(this.responseType){case YAHOO.widget.DS_XHR.TYPE_JSON:var jsonList,jsonObjParsed;var isNotMac=(navigator.userAgent.toLowerCase().indexOf("khtml")==-1);if(oResponse.parseJSON&&isNotMac){jsonObjParsed=oResponse.parseJSON();if(!jsonObjParsed){bError=true;}else{try{jsonList=eval("jsonObjParsed."+aSchema[0]);}catch(e){bError=true;break;}}}else{if(window.JSON&&isNotMac){jsonObjParsed=JSON.parse(oResponse);if(!jsonObjParsed){bError=true;break;}else{try{jsonList=eval("jsonObjParsed."+aSchema[0]);}catch(e){bError=true;break;}}}else{try{while(oResponse.substring(0,1)==" "){oResponse=oResponse.substring(1,oResponse.length);}if(oResponse.indexOf("{")<0){bError=true;break;}if(oResponse.indexOf("{}")===0){break;}var jsonObjRaw=eval("("+oResponse+")");if(!jsonObjRaw){bError=true;break;}jsonList=eval("(jsonObjRaw."+aSchema[0]+")");}catch(e){bError=true;break;}}}if(!jsonList){bError=true;break;}if(!YAHOO.lang.isArray(jsonList)){jsonList=[jsonList];}for(var i=jsonList.length-1;i>=0;i--){var aResultItem=[];var jsonResult=jsonList[i];for(var j=aSchema.length-1;j>=1;j--){var dataFieldValue=jsonResult[aSchema[j]];if(!dataFieldValue){dataFieldValue="";}aResultItem.unshift(dataFieldValue);}if(aResultItem.length==1){aResultItem.push(jsonResult);}aResults.unshift(aResultItem);}break;case YAHOO.widget.DS_XHR.TYPE_XML:var xmlList=oResponse.getElementsByTagName(aSchema[0]);
if(!xmlList){bError=true;break;}for(var k=xmlList.length-1;k>=0;k--){var result=xmlList.item(k);var aFieldSet=[];for(var m=aSchema.length-1;m>=1;m--){var sValue=null;var xmlAttr=result.attributes.getNamedItem(aSchema[m]);if(xmlAttr){sValue=xmlAttr.value;}else{var xmlNode=result.getElementsByTagName(aSchema[m]);if(xmlNode&&xmlNode.item(0)&&xmlNode.item(0).firstChild){sValue=xmlNode.item(0).firstChild.nodeValue;}else{sValue="";}}aFieldSet.unshift(sValue);}aResults.unshift(aFieldSet);}break;case YAHOO.widget.DS_XHR.TYPE_FLAT:if(oResponse.length>0){var newLength=oResponse.length-aSchema[0].length;if(oResponse.substr(newLength)==aSchema[0]){oResponse=oResponse.substr(0,newLength);}var aRecords=oResponse.split(aSchema[0]);for(var n=aRecords.length-1;n>=0;n--){aResults[n]=aRecords[n].split(aSchema[1]);}}break;default:break;}sQuery=null;oResponse=null;oParent=null;if(bError){return null;}else{return aResults;}};YAHOO.widget.DS_XHR.prototype._oConn=null;YAHOO.widget.DS_JSFunction=function(A,C){if(C&&(C.constructor==Object)){for(var B in C){this[B]=C[B];}}if(!YAHOO.lang.isFunction(A)){return ;}else{this.dataFunction=A;this._init();}};YAHOO.widget.DS_JSFunction.prototype=new YAHOO.widget.DataSource();YAHOO.widget.DS_JSFunction.prototype.dataFunction=null;YAHOO.widget.DS_JSFunction.prototype.doQuery=function(C,F,D){var B=this.dataFunction;var E=[];E=B(F);if(E===null){this.dataErrorEvent.fire(this,D,F,YAHOO.widget.DataSource.ERROR_DATANULL);return ;}var A={};A.query=decodeURIComponent(F);A.results=E;this._addCacheElem(A);this.getResultsEvent.fire(this,D,F,E);C(F,E,D);return ;};YAHOO.widget.DS_JSArray=function(A,C){if(C&&(C.constructor==Object)){for(var B in C){this[B]=C[B];}}if(!YAHOO.lang.isArray(A)){return ;}else{this.data=A;this._init();}};YAHOO.widget.DS_JSArray.prototype=new YAHOO.widget.DataSource();YAHOO.widget.DS_JSArray.prototype.data=null;YAHOO.widget.DS_JSArray.prototype.doQuery=function(E,I,A){var F;var C=this.data;var J=[];var D=false;var B=this.queryMatchContains;if(I){if(!this.queryMatchCase){I=I.toLowerCase();}for(F=C.length-1;F>=0;F--){var H=[];if(YAHOO.lang.isString(C[F])){H[0]=C[F];}else{if(YAHOO.lang.isArray(C[F])){H=C[F];}}if(YAHOO.lang.isString(H[0])){var G=(this.queryMatchCase)?encodeURIComponent(H[0]).indexOf(I):encodeURIComponent(H[0]).toLowerCase().indexOf(I);if((!B&&(G===0))||(B&&(G>-1))){J.unshift(H);}}}}else{for(F=C.length-1;F>=0;F--){if(YAHOO.lang.isString(C[F])){J.unshift([C[F]]);}else{if(YAHOO.lang.isArray(C[F])){J.unshift(C[F]);}}}}this.getResultsEvent.fire(this,A,I,J);E(I,J,A);};YAHOO.register("autocomplete",YAHOO.widget.AutoComplete,{version:"2.3.1",build:"541"});/*  Prototype JavaScript framework
 *  (c) 2005 Sam Stephenson <sam@conio.net>
 *  Prototype is freely distributable under the terms of an MIT-style license.
 *  For details, see the Prototype web site: http://prototype.conio.net/
/*--------------------------------------------------------------------------*/

//note: modified & stripped down version of prototype, to be used with moo.fx by mad4milk (http://moofx.mad4milk.net).

var Class = {
	create: function() {
		return function() {
			this.initialize.apply(this, arguments);
		}
	}
}

Object.extend = function(destination, source) {
	for (property in source) destination[property] = source[property];
	return destination;
}

Function.prototype.bind = function(object) {
	var __method = this;
	return function() {
		return __method.apply(object, arguments);
	}
}

Function.prototype.bindAsEventListener = function(object) {
var __method = this;
	return function(event) {
		__method.call(object, event || window.event);
	}
}

/**
 * @return {Element} 
 */
function $() {
	if (arguments.length == 1) return get$(arguments[0]);
	var elements = [];
	$c(arguments).each(function(el){
		elements.push(get$(el));
	});
	return elements;

	function get$(el){
		if (typeof el == 'string') el = document.getElementById(el);
		return el;
	}
}

if (!window.Element) var Element = new Object();

Object.extend(Element, {
	remove: function(element) {
		element = $(element);
		element.parentNode.removeChild(element);
	},

	hasClassName: function(element, className) {
		element = $(element);
		if (!element) return;
		var hasClass = false;
		element.className.split(' ').each(function(cn){
			if (cn == className) hasClass = true;
		});
		return hasClass;
	},

	addClassName: function(element, className) {
		element = $(element);
		Element.removeClassName(element, className);
		element.className += ' ' + className;
	},
  
	removeClassName: function(element, className) {
		element = $(element);
		if (!element) return;
		var newClassName = '';
		element.className.split(' ').each(function(cn, i){
			if (cn != className){
				if (i > 0) newClassName += ' ';
				newClassName += cn;
			}
		});
		element.className = newClassName;
	},

	cleanWhitespace: function(element) {
		element = $(element);
		$c(element.childNodes).each(function(node){
			if (node.nodeType == 3 && !/\S/.test(node.nodeValue)) Element.remove(node);
		});
	},

	find: function(element, what) {
		element = $(element)[what];
		while (element.nodeType != 1) element = element[what];
		return element;
	}
});

var Position = {
	cumulativeOffset: function(element) {
		var valueT = 0, valueL = 0;
		do {
			valueT += element.offsetTop  || 0;
			valueL += element.offsetLeft || 0;
			element = element.offsetParent;
		} while (element);
		return [valueL, valueT];
	}
};

document.getElementsByClassName = function(className) {
	var children = document.getElementsByTagName('*') || document.all;
	var elements = [];
	$c(children).each(function(child){
		if (Element.hasClassName(child, className)) elements.push(child);
	});  
	return elements;
}

//useful array functions
Array.prototype.each = function(func){
	for(var i=0;ob=this[i];i++) func(ob, i);
}

function $c(array){
	var nArray = [];
	for (i=0;el=array[i];i++) nArray.push(el);
	return nArray;
}/*
moo.fx, simple effects library built with prototype.js (http://prototype.conio.net).
by Valerio Proietti (http://mad4milk.net) MIT-style LICENSE.
for more info (http://moofx.mad4milk.net).
Sunday, March 05, 2006
v 1.2.3
*/

var fx = new Object();
//base
fx.Base = function(){};
fx.Base.prototype = {
	setOptions: function(options) {
	this.options = {
		duration: 500,
		onComplete: '',
		transition: fx.sinoidal
	}
	Object.extend(this.options, options || {});
	},

	step: function() {
		var time  = (new Date).getTime();
		if (time >= this.options.duration+this.startTime) {
			this.now = this.to;
			clearInterval (this.timer);
			this.timer = null;
			if (this.options.onComplete) setTimeout(this.options.onComplete.bind(this), 10);
		}
		else {
			var Tpos = (time - this.startTime) / (this.options.duration);
			this.now = this.options.transition(Tpos) * (this.to-this.from) + this.from;
		}
		this.increase();
	},

	custom: function(from, to) {
		if (this.timer != null) return;
		this.from = from;
		this.to = to;
		this.startTime = (new Date).getTime();
		this.timer = setInterval (this.step.bind(this), 13);
	},

	hide: function() {
		this.now = 0;
		this.increase();
	},

	clearTimer: function() {
		clearInterval(this.timer);
		this.timer = null;
	}
}

//stretchers
fx.Layout = Class.create();
fx.Layout.prototype = Object.extend(new fx.Base(), {
	initialize: function(el, options) {
		this.el = $(el);
		this.el.style.overflow = "hidden";
		this.iniWidth = this.el.offsetWidth;
		this.iniHeight = this.el.offsetHeight;
		this.setOptions(options);
	}
});

fx.Height = Class.create();
Object.extend(Object.extend(fx.Height.prototype, fx.Layout.prototype), {	
	increase: function() {
		this.el.style.height = this.now + "px";
	},

	toggle: function() {
		if (this.el.offsetHeight > 0) this.custom(this.el.offsetHeight, 0);
		else this.custom(0, this.el.scrollHeight);
	}
});

fx.Width = Class.create();
Object.extend(Object.extend(fx.Width.prototype, fx.Layout.prototype), {	
	increase: function() {
		this.el.style.width = this.now + "px";
	},

	toggle: function(){
		if (this.el.offsetWidth > 0) this.custom(this.el.offsetWidth, 0);
		else this.custom(0, this.iniWidth);
	}
});

//fader
fx.Opacity = Class.create();
fx.Opacity.prototype = Object.extend(new fx.Base(), {
	initialize: function(el, options) {
		this.el = $(el);
		this.now = 1;
		this.increase();
		this.setOptions(options);
	},

	increase: function() {
		if (this.now == 1 && (/Firefox/.test(navigator.userAgent))) this.now = 0.9999;
		this.setOpacity(this.now);
	},
	
	setOpacity: function(opacity) {
		if (opacity == 0 && this.el.style.visibility != "hidden") this.el.style.visibility = "hidden";
		else if (this.el.style.visibility != "visible") this.el.style.visibility = "visible";
		//if (window.ActiveXObject) {
			this.el.style.filter = "alpha(opacity=" + opacity*100 + ")";
		//}
		this.el.style.opacity = opacity;
	},

	toggle: function() {
		if (this.now > 0) this.custom(1, 0);
		else this.custom(0, 1);
	}
});

//transitions
fx.sinoidal = function(pos){
	return ((-Math.cos(pos*Math.PI)/2) + 0.5);
	//this transition is from script.aculo.us
}
fx.linear = function(pos){
	return pos;
}
fx.cubic = function(pos){
	return Math.pow(pos, 3);
}
fx.circ = function(pos){
	return Math.sqrt(pos);
}/*
moo.fx pack, effects extensions for moo.fx.
by Valerio Proietti (http://mad4milk.net) MIT-style LICENSE
for more info visit (http://moofx.mad4milk.net).
Tuesday, March 07, 2006
v 1.2.3
*/

//smooth scroll
fx.Scroll = Class.create();
fx.Scroll.prototype = Object.extend(new fx.Base(), {
	initialize: function(options) {
		this.setOptions(options);
	},

	scrollTo: function(el){
		var dest = Position.cumulativeOffset($(el))[1];
		var client = YAHOO.util.Dom.getClientHeight();
		//var full = document.documentElement.scrollHeight;
		var full = OZONE.visuals.bodyHeight();
		var top = window.pageYOffset || document.body.scrollTop || document.documentElement.scrollTop;
		if (dest+client > full) this.custom(top, dest - client + (full-dest));
		else this.custom(top, dest);
	},

	increase: function(){
		window.scrollTo(0, this.now);
	}
});

//smooth scroll to center the element
fx.ScrollCenter = Class.create();
fx.ScrollCenter.prototype = Object.extend(new fx.Base(), {
	initialize: function(options) {
		this.setOptions(options);
	},

	scrollTo: function(el){
		var elHeight = $(el).offsetHeight;
		//alert(elHeight);
		var dest = Position.cumulativeOffset($(el))[1];
		//var client = window.innerHeight || document.documentElement.clientHeight;
		var client = YAHOO.util.Dom.getClientHeight();
		if(elHeight<client){
			dest = Math.max(dest-client*0.5 + elHeight*0.5,0);
		}
		//var full = document.documentElement.scrollHeight;
		var full = OZONE.visuals.bodyHeight();
		var top = window.pageYOffset || document.body.scrollTop || document.documentElement.scrollTop;
		if (dest+client > full) this.custom(top,full-client);
		else this.custom(top, dest);
	},

	increase: function(){
		window.scrollTo(0, this.now);
	}
});

//smooth scroll to center the element
fx.ScrollBottom = Class.create();
fx.ScrollBottom.prototype = Object.extend(new fx.Base(), {
	initialize: function(options) {
		this.setOptions(options);
	},

	scrollTo: function(el){
		var dest = Position.cumulativeOffset($(el))[1];
		var client = YAHOO.util.Dom.getClientHeight();
		var dest = Math.max(dest-client+40,0);
		var full = OZONE.visuals.bodyHeight();
		var top = window.pageYOffset || document.body.scrollTop || document.documentElement.scrollTop;
		if (dest+client > full) this.custom(top, dest - client + (full-dest));
		else this.custom(top, dest);
	},

	increase: function(){
		window.scrollTo(0, this.now);
	}
});

//text size modify, now works with pixels too.
fx.Text = Class.create();
fx.Text.prototype = Object.extend(new fx.Base(), {
	initialize: function(el, options) {
		this.el = $(el);
		this.setOptions(options);
		if (!this.options.unit) this.options.unit = "em";
	},

	increase: function() {
		this.el.style.fontSize = this.now + this.options.unit;
	}
});

//composition effect: widht/height/opacity
fx.Combo = Class.create();
fx.Combo.prototype = {
	setOptions: function(options) {
		this.options = {
			opacity: true,
			height: true,
			width: false
		}
		Object.extend(this.options, options || {});
	},

	initialize: function(el, options) {
		this.el = $(el);
		this.setOptions(options);
		if (this.options.opacity) {
			this.o = new fx.Opacity(el, options);
			options.onComplete = null;
		}
		if (this.options.height) {
			this.h = new fx.Height(el, options);
			options.onComplete = null;
		}
		if (this.options.width) this.w = new fx.Width(el, options);
	},
	
	toggle: function() { this.checkExec('toggle'); },

	hide: function(){ this.checkExec('hide'); },
	
	clearTimer: function(){ this.checkExec('clearTimer'); },
	
	checkExec: function(func){
		if (this.o) this.o[func]();
		if (this.h) this.h[func]();
		if (this.w) this.w[func]();
	},
	
	//only if width+height
	resizeTo: function(hto, wto) {
		if (this.h && this.w) {
			this.h.custom(this.el.offsetHeight, this.el.offsetHeight + hto);
			this.w.custom(this.el.offsetWidth, this.el.offsetWidth + wto);
		}
	},

	customSize: function(hto, wto) {
		if (this.h && this.w) {
			this.h.custom(this.el.offsetHeight, hto);
			this.w.custom(this.el.offsetWidth, wto);
		}
	}
}

fx.Accordion = Class.create();
fx.Accordion.prototype = {
	setOptions: function(options) {
		this.options = {
			delay: 100,
			opacity: false
		}
		Object.extend(this.options, options || {});
	},

	initialize: function(togglers, elements, options) {
		this.elements = elements;
		this.setOptions(options);
		var options = options || '';
		elements.each(function(el, i){
			options.onComplete = function(){
				if (el.offsetHeight > 0) el.style.height = '1%';
			}
			el.fx = new fx.Combo(el, options);
			el.fx.hide();
		});

		togglers.each(function(tog, i){
			tog.onclick = function(){
				this.showThisHideOpen(elements[i]);
			}.bind(this);
		}.bind(this));
	},

	showThisHideOpen: function(toShow){
		this.elements.each(function(el, i){
			if (el.offsetHeight > 0 && el != toShow) this.clearAndToggle(el);
		}.bind(this));
		if (toShow.offsetHeight == 0) setTimeout(function(){this.clearAndToggle(toShow);}.bind(this), this.options.delay);

	},

	clearAndToggle: function(el){
		el.fx.clearTimer();
		el.fx.toggle();
	}
}

var Remember = new Object();
Remember = function(){};
Remember.prototype = {
	initialize: function(el, options){
		this.el = $(el);
		this.days = 365;
		this.options = options;
		this.effect();
		var cookie = this.readCookie();
		if (cookie) {
			this.fx.now = cookie;
			this.fx.increase();
		}
	},

	//cookie functions based on code by Peter-Paul Koch
	setCookie: function(value) {
		var date = new Date();
		date.setTime(date.getTime()+(this.days*24*60*60*1000));
		var expires = "; expires="+date.toGMTString();
		document.cookie = this.el+this.el.id+this.prefix+"="+value+expires+"; path=/";
	},

	readCookie: function() {
		var nameEQ = this.el+this.el.id+this.prefix + "=";
		var ca = document.cookie.split(';');
		for(var i=0;c=ca[i];i++) {
			while (c.charAt(0)==' ') c = c.substring(1,c.length);
			if (c.indexOf(nameEQ) == 0) return c.substring(nameEQ.length,c.length);
		}
		return false;
	},

	custom: function(from, to){
		if (this.fx.now != to) {
			this.setCookie(to);
			this.fx.custom(from, to);
		}
	}
}

fx.RememberHeight = Class.create();
fx.RememberHeight.prototype = Object.extend(new Remember(), {
	effect: function(){
		this.fx = new fx.Height(this.el, this.options);
		this.prefix = 'height';
	},
	
	toggle: function(){
		if (this.el.offsetHeight == 0) this.setCookie(this.el.scrollHeight);
		else this.setCookie(0);
		this.fx.toggle();
	},
	
	resize: function(to){
		this.setCookie(this.el.offsetHeight+to);
		this.fx.custom(this.el.offsetHeight,this.el.offsetHeight+to);
	},

	hide: function(){
		if (!this.readCookie()) {
			this.fx.hide();
		}
	}
});

fx.RememberText = Class.create();
fx.RememberText.prototype = Object.extend(new Remember(), {
	effect: function(){
		this.fx = new fx.Text(this.el, this.options);
		this.prefix = 'text';
	}
});

//useful for-replacement
Array.prototype.each = function(func){
	for(var i=0;ob=this[i];i++) func(ob, i);
}

//Easing Equations (c) 2003 Robert Penner, all rights reserved.
//This work is subject to the terms in http://www.robertpenner.com/easing_terms_of_use.html.

//expo
fx.expoIn = function(pos){
	return Math.pow(2, 10 * (pos - 1));
}
fx.expoOut = function(pos){
	return (-Math.pow(2, -10 * pos) + 1);
}

//quad
fx.quadIn = function(pos){
	return Math.pow(pos, 2);
}
fx.quadOut = function(pos){
	return -(pos)*(pos-2);
}

//circ
fx.circOut = function(pos){
	return Math.sqrt(1 - Math.pow(pos-1,2));
}
fx.circIn = function(pos){
	return -(Math.sqrt(1 - Math.pow(pos, 2)) - 1);
}

//back
fx.backIn = function(pos){
	return (pos)*pos*((2.7)*pos - 1.7);
}
fx.backOut = function(pos){
	return ((pos-1)*(pos-1)*((2.7)*(pos-1) + 1.7) + 1);
}

//sine
fx.sineOut = function(pos){
	return Math.sin(pos * (Math.PI/2));
}
fx.sineIn = function(pos){
	return -Math.cos(pos * (Math.PI/2)) + 1;
}
fx.sineInOut = function(pos){
	return -(Math.cos(Math.PI*pos) - 1)/2;
}