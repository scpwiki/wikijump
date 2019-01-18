OZONE.dom.onDomReady(function(){		
	// change links to http://...
	var els = document.getElementsByTagName('a');
	for(var i=0; i<els.length;i++){
		els[i].href = els[i].href.replace(/^https/, 'http');
	}
}, "dummy-ondomready-block");