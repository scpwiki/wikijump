dojo.require("dojo.parser");

dojo.require("dijit.Dialog");
dojo.require("dijit.form.Button");
dojo.require("dijit.layout.LayoutContainer");
dojo.require("dijit.layout.ContentPane");
dojo.require("demos.flashCards.src.MathFlashCard");
dojo.require("demos.flashCards.src.Teacher");

//global storage point for our fade animations
var messageFadeEvent = null;

//reset the score board
var resetScore = function(){
	var card = dijit.byId("card");
	dojo.byId("remainingTd").innerHTML = card.numberProblems;
	dojo.byId("correctTd").innerHTML = 0;
	dojo.byId("incorrectTd").innerHTML = 0;
	dojo.byId("tooSlowTd").innerHTML = 0;
	dojo.style("scoreTable", "display", "");
};

var goodAnswers = ["Great job. Did you know that doing math regularly keeps you sharp?", "Hey, this is correct, did you ever concider joining a math competition?", "I am impressed! Not so many students are good in math, congratulations!"];
var indexGood = 0;
var countGood = 0;

var badAnswers = ["Hmm, you are letting me down! Did you practice enough?", "This is wrong, why don't you know such a simple answer?", "I might have to send you back to school, this is not good"];
var indexBad = 0;

var slowAnswers = ["Hmm, I thought you could have beena little faster to be honest", "You could have counted the result on your fingers my friend.", "Slow slow slow, what are we going to do about this?"];
var indexSlow = 0;

var countBad = 0;

//start the flashcards, result of pressing "getStarted"
var getStarted = function(){
	dijit.byId("teacher").speak("Hi student! Let's go. Just enter the correct results and press enter.");
	
	//call the card widgets getStarted() method
	var c = dijit.byId("card");
	c.getStarted();

	//display(unhide) the card
	dojo.style(c.domNode,"display","");

	//hide the getStarted Button
	dojo.style("start", "display", "none");

	//reset the score board	
	resetScore();

	//attach to the onCorrect event of the card.  When this event happens
	//we want to display a fading message and update the score board				
	dojo.connect(c, "onCorrect", function(e){
		if (messageFadeEvent){
			messageFadeEvent.stop();
		}
		
		countGood++;
		countBad = 0;
		if (countGood == 2){
			dijit.byId("teacher").frown(false);
		}

		dojo.byId("correctTd").innerHTML = c.correctAnswers;

		if (c.started){
			dojo.byId("remainingTd").innerHTML = c.numberProblems - c.currentProblem;
			// FIXME: which node?
			dojo.removeClass("incorrect");
			dojo.addClass("correct");
			
			dijit.byId("teacher").speak(goodAnswers[indexGood], 5000);
			indexGood++;
			if (indexGood == goodAnswers.length) { indexGood=0; }
		}
	});

	//attach to the onIncorrect event of the card.  When this event happens
	//we want to display a fading message and update the score board				
	dojo.connect(c, "onIncorrect", function(e){
		if(messageFadeEvent){
			messageFadeEvent.stop();
		}
		
		countBad++;
		countGood = 0;
		if (countBad == 2){
			dijit.byId("teacher").frown(true);
		}

		dojo.byId("incorrectTd").innerHTML = c.incorrectAnswers;

		if(c.started){
			dojo.byId("remainingTd").innerHTML = c.numberProblems - c.currentProblem;
			dojo.addClass("incorrect");
			dojo.removeClass("correct");
			
			dijit.byId("teacher").speak(badAnswers[indexBad], 5000);
			indexBad++;
			if (indexBad == badAnswers.length) { indexBad=0; }
		}
	});

	//attach to the onTooSlow event of the card.  When this event happens
	//we want to display a fading message and update the score board				
	dojo.connect(c, "onTooSlow", function(e){
		if(messageFadeEvent){
			messageFadeEvent.stop();
		}
		
		countBad++;
		countGood = 0;
		if (countBad == 2){
			dijit.byId("teacher").frown(true);
		}

		dojo.byId("tooSlowTd").innerHTML = c.tooSlow;

		if(c.started){
			dojo.byId("remainingTd").innerHTML = c.numberProblems - c.currentProblem;
			// FIXME: which node?
			dojo.addClass("incorrect");
			dojo.removeClass("correct");
			
			dijit.byId("teacher").speak(slowAnswers[indexSlow], 5000);
			indexSlow++;
			if (indexSlow == slowAnswers.length) { indexSlow=0; }
		}
		
	});

	//attach to the onEnd event of the card.  When this event happens
	//we want to display a final message,update the score board, and show
	//the restart button
		
	dojo.connect(c, "onEnd", function(correct, total, slow){
		if (messageFadeEvent){
			messageFadeEvent.stop();
		}

		//create ending message/summary
		var msg = "You got " + correct + " out of " + total + " correct.";
		if(slow > 0){
			msg += " You were too slow on ";
			if(slow == 1){
				msg += "1 problem.";
			} else {
				msg += slow + " problems.";
			}
		}
		
		if (c.correctAnswers > 8){
			dijit.byId("teacher").frown(false);
		}

		dijit.byId("teacher").speak(msg);
		
		dojo.style(c.domNode, "display", "none");
		dojo.style(dojo.byId("restart"), "display", "");
		dojo.byId("remainingTd").innerHTML = "0";

	});

	//attach to the restart method of the card.  We dont' want to reset the 
	//score onEnd since people want a chance to review their score, this won't
	//happen until they actually click restart
	dojo.connect(c, "restart", resetScore);
};

//the restart function
var restart = function(){
	dojo.style(dojo.byId("restart"),"display","none");
	dojo.style(dijit.byId("card").domNode,"display","");
	dijit.byId("teacher").speak("What a delightful student you are, keep practicing! Your way to success.");
	dijit.byId("card").restart();
};

dojo.addOnLoad(function(){
	
	dojo.parser.parse();
	
	setInterval(function(){
		dijit.byId("teacher").blink();
	}, 2500);
});