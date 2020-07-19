

Wikijump.modules.FlickrGalleryModules = {};

Wikijump.modules.FlickrGalleryModules.vars = {};

Wikijump.modules.FlickrGalleryModules.listeners = {
	loadPage: function(event,pageNumber){
		// get gallery box
		var box = YAHOO.util.Event.getTarget(event);
		do{
			box = box.parentNode;
		}while(box && !YAHOO.util.Dom.hasClass(box, 'flickr-gallery-box'));
		Wikijump.modules.FlickrGalleryModules.vars.destinationBox = box;

		var p = new Object();

		// get original parameters
		var ps = YAHOO.util.Dom.getElementsByClassName("flickr-gallery-parameter", 'li', box);
		for(var i=0; i<ps.length; i++){
			var vals = ps[i].innerHTML.split(':::');
			p[vals[0].replace(/^ +/,'').replace(/ +$/, '')] =  vals[1].replace(/^ +/,'').replace(/ +$/, '');
		}
		p.pageNumber = pageNumber;
		p.contentOnly = true;

		OZONE.ajax.requestModule("wiki/image/FlickrGalleryModule", p, Wikijump.modules.FlickrGalleryModules.callbacks.loadPage);
	},

	showPhoto: function(e, photoId){
		YAHOO.util.Event.stopEvent(e);
		p = new Object();
		p.photoId = photoId;
		OZONE.ajax.requestModule("wiki/image/FlickrGalleryViewPhotoModule", p, Wikijump.modules.FlickrGalleryModules.callbacks.showPhoto);
		var box = YAHOO.util.Event.getTarget(e);
		do{
			box = box.parentNode;
		}while(box && !YAHOO.util.Dom.hasClass(box, 'flickr-gallery-box'))
		var ps = YAHOO.util.Dom.getElementsByClassName("flickr-gallery-order", 'li', box);

		var galleryList = new Array();
		var currentElement = 0;
		for(var i=0; i<ps.length; i++){
			galleryList.push(0+ps[i].innerHTML);
			if(1*ps[i].innerHTML == 1*photoId){
				currentElement = i;
			}
		}

		Wikijump.modules.FlickrGalleryModules.vars.currentElement = currentElement;
		Wikijump.modules.FlickrGalleryModules.vars.galleryList = galleryList;

	},

	showPreviousPhoto: function(e){
		var ce = Wikijump.modules.FlickrGalleryModules.vars.currentElement;
		var gl = Wikijump.modules.FlickrGalleryModules.vars.galleryList;
		if(ce>0){
			Wikijump.modules.FlickrGalleryModules.listeners.showPhoto(e, gl[ce-1]);
		}
	},
	showNextPhoto: function(e){
		var ce = Wikijump.modules.FlickrGalleryModules.vars.currentElement;
		var gl = Wikijump.modules.FlickrGalleryModules.vars.galleryList;
		if(ce<gl.length-1){
			Wikijump.modules.FlickrGalleryModules.listeners.showPhoto(e, gl[ce+1]);
		}
	}

}

Wikijump.modules.FlickrGalleryModules.callbacks = {
	loadPage: function(r){
		if(!Wikijump.utils.handleError(r)) {return;}
		var box = Wikijump.modules.FlickrGalleryModules.vars.destinationBox;
		box.getElementsByTagName('div')[0].innerHTML = r.body;
		var els = box.getElementsByTagName('img');//YAHOO.util.Dom.getElementsByClassName('gallery-item', 'div', boxes[i]);
		OZONE.dialog.hovertip.makeTip(els, {style: {width: 'auto'}, noCursorHelp: true, delay: 50});
	},

	showPhoto: function(r){
		if(!Wikijump.utils.handleError(r)) {return;}

		// if window already there...
		var photowindow = YAHOO.util.Dom.getElementsByClassName('photowindow', 'div', 'odialog-container')[0];
		if(photowindow){
			var inner = r.body.replace('<div class="owindow photowindow" id="photowindow">', '');
			inner = inner.replace('</div><!-- end -->', '');
			inner = inner.replace('<img class="flickrphoto"', '<img class="flickrphoto" style="visibility:hidden"');
			photowindow.innerHTML = inner;
			var image = YAHOO.util.Dom.getElementsByClassName('flickrphoto', 'img', photowindow)[0];
			var eff = new fx.Opacity(image, {duration: 400});
			eff.setOpacity(0);
			eff.custom(0,1);
			OZONE.dialog.factory.boxcontainer().centerContent();
		}else{

			var w = new OZONE.dialogs.Dialog();
			w.content = r.body;
			w.clickOutsideToClose = true;
			w.smooth = true;
			w.show();
		}

		// show arrows?
		var ce = Wikijump.modules.FlickrGalleryModules.vars.currentElement;
		var gl = Wikijump.modules.FlickrGalleryModules.vars.galleryList;
		if(ce == 0){
			$("photo-nav-prev").style.visibility="hidden";
		}else{
			$("photo-nav-prev").style.visibility="visible";
		}
		if(ce == gl.length-1){
			$("photo-nav-next").style.visibility="hidden";
		}else{
			$("photo-nav-next").style.visibility="visible";
		}

	}

}

Wikijump.modules.FlickrGalleryModules.init = function(){

	OZONE.dom.onDomReady(function(){
		var boxes = YAHOO.util.Dom.getElementsByClassName('flickr-gallery-box', 'div');
		for(var i = 0; i<boxes.length; i++){
			if(YAHOO.util.Dom.hasClass(boxes[i], 'makeHoverTitles')){
				var els = boxes[i].getElementsByTagName('img');//YAHOO.util.Dom.getElementsByClassName('gallery-item', 'div', boxes[i]);
				OZONE.dialog.hovertip.makeTip(els, {style: {width: 'auto'}, noCursorHelp: true, delay: 50});
			}
		}
	}, "dummy-ondomready-block");
}

Wikijump.modules.FlickrGalleryModules.init();
