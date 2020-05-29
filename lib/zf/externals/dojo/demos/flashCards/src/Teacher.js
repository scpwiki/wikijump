dojo.provide("demos.flashCards.src.Teacher");

dojo.require("dijit._Widget");
dojo.require("dijit._Templated");
dojo.require("dojox.timing.Sequence");

dojo.declare("demo._TeacherBehavior", null, {
	
	constructor: function(){
		this._batEyes = [
			{ func: [dojo.hitch(this,"angry",true), this], pauseAfter:200 },
			{ func: [dojo.hitch(this,"blink",45), this], repeat: 3, pauseAfter: 100 },
			{ func: [dojo.hitch(this,"angry",false), this], pauseBefore:200 }
		];
		
	},
	
	bat: function(){
		this.actions.go(this._batEyes);
	}
		
});

dojo.declare("demo.Teacher", [dijit._Widget, dijit._Templated, demo._TeacherBehavior], {

	templatePath: dojo.moduleUrl("demos.flashCards.src","Teacher.html"),

	constructor: function(args, node){
		dojo.mixin(this, args);
		this.actions = new dojox.timing.Sequence({});
	},
	
	frown: function(/* Boolean */on){
		var n = this.innerNode;
		dojo[(on ? "addClass" : "removeClass")](n,"teacherBeingAngry");
	},
	
	blink: function(closeDuration, forced){
		if(this._blinking && !forced){ clearTimeout(this._blinking); }
		dojo.addClass(this.innerNode, "teacherBlinking");
		this._blinking = setTimeout(dojo.hitch(this,function(){
			dojo.removeClass(this.innerNode,"teacherBlinking");
		}), closeDuration || 275);
	},
	
	speak: function(speach, timeout){
		if (!timeout) { timeout = 5000; }
		
		this.teacherBubble.innerHTML = speach;
		dojo.addClass(this.teacherBubbleOuter, "teacherSpeaking");
		
		if (this._timeout) { clearTimeout(this._timeout); }
		this._timeout = setTimeout(dojo.hitch(this, function(){
			dojo.removeClass(this.teacherBubbleOuter, "teacherSpeaking")
		}), timeout);
	}
	
});
