dojo.require("dijit.layout.AccordionContainer");
dojo.require("dojox.layout.ScrollPane");
dojo.require("dojox.widget.FisheyeLite");
//dojo.require("dojox.layout.DragPane");
dojo.require("dojo.NodeList-fx");
dojo.require("dojo.fx");
dojo.require("dojo.fx.easing");

var show = null;
(function($){

	show = function(id){
		var contents = dojo.byId(id).innerHTML;
		$("#content").style("opacity", 0).forEach(function(n){ n.innerHTML = contents; }).anim({ opacity:1 });
	}
	
	var init = function(){
			// turn li's in this page into fisheye items, presumtiously:  
		 $("#hidden ul > li").forEach(function(n){
			new dojox.widget.FisheyeLite({
				properties:{
				  fontSize:1.5
				},
				easeIn: dojo.fx.easing.linear,
				durationIn: 100,
				easeOut: dojo.fx.easing.linear,
				durationOut: 100
			  }, n);
		  });
	
		panes = [];
		var paneType = dojo.getObject("dojox.layout.ScrollPane");
	
		dojo.forEach(["day1s","day2s","day3s"], function(id){
			var scroll = new paneType({
				  style: "width:450px; height:170px"
			}, id);
			scroll.startup();
			panes.push(scroll);			
		});
	
		//accordion widget
		accordion = new dijit.layout.AccordionContainer({}, "accordionPanel");
		var content1 = new dijit.layout.AccordionPane({ id:'pane1', title: '25.07.2008', selected: true },'day1').placeAt(accordion);
		var content2 = new dijit.layout.AccordionPane({ id:'pane2', title: '26.07.2008' }, 'day2').placeAt(accordion);
		var content3 = new dijit.layout.AccordionPane({ id:'pane3', title: '27.07.2008' }, 'day3').placeAt(accordion);
	
		dojo.forEach([content1, content2, content3], function(pane,i){
			// store a ref to the scrollpane contained within each accordionpane
			dojo.mixin(pane, {
				innerScroller: panes[i]
			});
		});

		dojo.subscribe("accordionPanel-selectChild", function(child){
			setTimeout(dojo.hitch(child.innerScroller,"layout"), 300);
		});

		accordion.startup();
		content1.innerScroller.layout();
	
		$('.dijitAccordionText').style('opacity', 0.01);
	}

	dojo.addOnLoad(init);

})(dojo.query);