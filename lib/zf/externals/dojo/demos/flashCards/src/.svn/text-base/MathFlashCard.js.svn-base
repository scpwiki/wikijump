dojo.provide("demos.flashCards.src.MathFlashCard");

dojo.require("dijit._Widget");
dojo.require("dijit._Templated");

dojo.declare("dojofun.widget.MathFlashCard",
	[dijit._Widget, dijit._Templated],
	{
		numberProblems: 20,
		minInt: 0,
		maxInt: 10,
		timer: 15,
		betweenTime: 5,
		problemType: "addition",
		currentProblem: 0,
		correctAnswers: 0,
		incorrectAnswers: 0,
		problemSet: null,
		started: false,
		tooSlow: 0,

		templatePath: dojo.moduleUrl("demos.flashCards","src/MathFlashCard.html"),

		generateProblemSet: function(){
			var problems=[];

			if (this.problemType=="addition"){
				for (var i=0; i<this.numberProblems; i++){
					problems.push({
						x: Math.floor(Math.random()*(this.maxInt+1)),
						y: Math.floor(Math.random()*(this.maxInt+1)),
						type: "addition"
					});
				}
			}

			return problems;
		}, 

		restart: function(){
			this.problemSet = this.generateProblemSet();
			dojo.forEach(["currentProblem","correctAnswers","incorrectAnswers","tooSlow"], 
				function(attr){
					this[attr] = 0;
			}, this)
			this.renderProblem(0);
			this.getStarted();
		},

		renderProblem: function(prob){
			this.topR.innerHTML = this.problemSet[prob].x;
			this.bottomR.innerHTML = this.problemSet[prob].y;
			this.answerInput.value = "";	
			if (this.problemSet[prob].type == "addition"){
				this.bottomL.innerHTML = "+";
			}

			if (this.started){
				this.getStarted();
			}

		},

		getStarted: function(){
			dojo.style(this.domNode, "opacity", 100);
			this.fadeOp = dojo.fadeOut({
				node: this.domNode,
				duration: (this.timer * 1000),
				onEnd: dojo.hitch(this, "onTooSlow") 
			}).play();

			if (!this.started){
				this.started = true;
				var func = dojo.hitch(dijit,"focus", this.answerInput);
				this.connect(this.answerInput,"onblur", function(){
					setTimeout(func, 10);
				});
				setTimeout(func, 10);
				this.listenForKeyEvents();
			}
		},

		listenForKeyEvents: function(){

			this.onKeyDownEvent = this.connect(dojo.doc,"onkeydown", function(e){
				var key = e.keyCode || e.charCode;
				var k = dojo.keys;
				switch(key){
					case k.ENTER:
						if(this.checkAnswer()){
							this.onCorrect();
						}else{
							this.onIncorrect();
						}
						break;
				}
			});
		},

		stopListening: function(){
			this.disconnect(this.onKeyDownEvent);
			delete this.onKeyDownEvent;
		},

		checkAnswer: function(){
			var prob = this.problemSet[this.currentProblem];

			if(prob.type=="addition"){
				var correct = prob.x + prob.y;
			}
			console.log(this.answerInput.value,"value");
			if (correct && this.answerInput.value==correct){
				return true;
			}
		},

		onCorrect: function(){
			this.correctAnswers++;
			this.fadeOp.stop();
			delete this.fadeOp;
			if ((this.currentProblem+1)<this.problemSet.length){
				this.renderProblem(++this.currentProblem);
			} else {
				this.onEnd(this.correctAnswers,this.problemSet.length,this.tooSlow);
			}

		},	

		onIncorrect: function(){
			this.incorrectAnswers++;
			this.fadeOp.stop();
			delete this.fadeOp;
			if ((this.currentProblem+1)<this.problemSet.length){
				this.renderProblem(++this.currentProblem);
			} else {
				this.onEnd(this.correctAnswers,this.problemSet.length,this.tooSlow);
			}
		},

		onTooSlow: function(){
			this.tooSlow++;
			if ((this.currentProblem+1)<this.problemSet.length){
				this.renderProblem(++this.currentProblem);
			} else {
				this.onEnd(this.correctAnswers,this.problemSet.length,this.tooSlow);
			}
		},

		onEnd: function(correct, set, tooSlow){
				this.stopListening();
				this.started=false;
		},

		postCreate:function(){
			this.problemSet = this.generateProblemSet();
			this.renderProblem(0);
		}
	}
);
