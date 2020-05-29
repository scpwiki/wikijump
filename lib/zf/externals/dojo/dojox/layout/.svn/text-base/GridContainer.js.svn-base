dojo.provide("dojox.layout.GridContainer");
dojo.experimental("dojox.layout.GridContainer");

dojo.require("dijit._base.focus");
dojo.require("dijit._Templated");
dojo.require("dijit._Container");
dojo.require("dojo.dnd.move");
dojo.require("dojox.layout.dnd.PlottedDnd");

dojo.requireLocalization("dojox.layout", "GridContainer");

dojo.declare("dojox.layout.GridContainer", 
	[dijit._Widget, dijit._Templated, dijit._Container, dijit._Contained], 
	{
	// summary:
	//		The Grid Container is a container of child elements that are placed in a kind of grid.
	//
	// description:
	// 		It displays the child elements by column (ie: the childs widths are fixed by the column width of the grid but the childs heights are free).
	//		Each child is movable by drag and drop inside the Grid Container.
	//		The position of other children is automatically calculated when a child is moved
	//	

	templatePath: dojo.moduleUrl("dojox.layout", "resources/GridContainer.html"),
	isContainer: true,

	//	i18n: Object
	//		Contain i18n ressources.
	i18n: null,
	
	//isAutoOrganized: Boolean:
	//	Define auto organisation of children into the grid container.
	isAutoOrganized : true,
	
	//isRightFixed: Boolean
	//	Define if the right border has a fixed size.
	isRightFixed:false,
	
	//isLeftFixed: Boolean
	//	Define if the left border has a fixed size.
	isLeftFixed:false,
	
	// hasResizableColumns: Boolean
	//	Allow or not resizing of columns by a grip handle.
	hasResizableColumns:true,
	
	// nbZones: Integer
	//		The number of dropped zones.
	nbZones:1,

	//opacity: Integer
	//		Define the opacity of the DnD Avatar.
	opacity:1,

	// minColWidth: Integer
	//		Minimum column width in percentage.
	minColWidth: 20,
 
	// minChildWidth: Integer
	// 		Minimun children with in pixel (only used for IE6 that doesn't handle min-width css property */
	minChildWidth : 150,

	// acceptTypes: Array
	//		The gridcontainer will only accept the children that fit to the types.
	//		In order to do that, the child must have a widgetType or a dndType attribute corresponding to the accepted type.*/
	acceptTypes: [], 
	
	//mode: String
	// 		location to add columns, must be set to left or right(default)
	mode: "right",

	//allowAutoScroll: Boolean
	//	auto-scrolling enable inside the GridContainer 
	allowAutoScroll: false,

	//timeDisplayPopup: Integer
	// 	display time of popup in miliseconds	
	timeDisplayPopup: 1500,

	//isOffset: Boolean 
    // 	if true : Let the mouse to its original location when moving (allow to specify it proper offset) 
    // 	if false : Current behavior, mouse in the upper left corner of the widget
  	isOffset: false,

	//offsetDrag: Object
	//	 Allow to specify its own offset (x and y) onl when Parameter isOffset is true
  	offsetDrag : {}, // 

	//withHandles: Boolean
	//	Specify if there is a specific drag handle on widgets	
	withHandles: false,

	// handleClasses: Array
	//	Array of classes of nodes that will act as drag handles
	handleClasses : [],
	
	//Contains the DnD widget
	_draggedWidget: null,

	//_isResized: Boolean
	//	Determine if user can resizing the widget with the mouse.
	_isResized: false,

	//Contains the node grip to resize widget.
	_activeGrip: null,

	//_oldwidth: Integer
	//	Save the old width size.
	_oldwidth: 0,

	//_oldheight: Integer
	//	Save the old height size.
	_oldheight: 0,	

	// a11y with keyboard is On/Off 
	_a11yOn : false,

	// can display a popup 
	_canDisplayPopup : true,

	constructor: function(props, node){
		// FIXME: does this need a "scopeName"
		this.acceptTypes = props["acceptTypes"] || ["dijit.layout.ContentPane"];
		this.dragOffset = props["dragOffset"] || { x:0, y:0 };
	},

	postMixInProperties: function(){
		this.i18n = dojo.i18n.getLocalization("dojox.layout", "GridContainer"); 
	},
	
	_createCells: function() {
		if(this.nbZones === 0){ this.nbZones = 1; }
		var wCol = 100 / this.nbZones;
		if(dojo.isIE && dojo.marginBox(this.gridNode).height){
			var space = document.createTextNode(" ");
			this.gridNode.appendChild(space);
		}
		var grid = [];
		this.cell = [];
		var i = 0;
		while(i < this.nbZones){
			var node = dojo.doc.createElement("td");
			dojo.addClass(node, "gridContainerZone");
			node.id = this.id + "_dz" + i;
			node.style.width = wCol + "%";	
			var zone = this.gridNode.appendChild(node);
			this.cell[i] = zone;
			i++;
		}
	},
	
	startup:function(){
		this._createCells();
		if(this.usepref !== true){
			this[(this.isAutoOrganized ? "_organizeServices" : "_organizeServicesManually")]();
		}else{ 
			//console.info("GridContainer organised by UserPref");
			return;
		}
		this.init();	
	},
	
	init: function(){
		// summary: Initialization of the GridContainer widget	
		this.grid = this._createGrid();
		this.connect(dojo.global, "onresize", "onResized");
		this.connect(this, "onDndDrop", "_placeGrips");
		this.dropHandler= dojo.subscribe("/dnd/drop", this, "_placeGrips");
		this._oldwidth = this.domNode.offsetWidth;
		if(this.hasResizableColumns){
			this._initPlaceGrips();
			this._placeGrips();			
		}
	},
	
	destroy: function(){
		// summary: destroy GridContainer Component.
		for(var i = 0; i < this.handleDndStart; i++){
			dojo.disconnect(this.handleDndStart[i]);
		}
		dojo.unsubscribe(this.dropHandler);
		this.inherited(arguments);
	},
	
/*	FIXME: implement resize / BorderContainer support
	resize: function(){
		console.log('resize',arguments)
	},
*/
	
	onResized: function(){
		// summary: Callback method to resize the GridContainer widget and columns
		if(this.hasResizableColumns){
			this._placeGrips();
			this._oldwidth = this.domNode.offsetWidth;
			this._oldheight = this.domNode.offsetHeight;
		}
	},
	
	/***********services methods******************/
	_organizeServices : function(){
		//summary: List all zones and insert service into columns.	
		var nbz = this.nbZones;	
		var nbs = this.getChildren().length;
		var res = Math.floor(nbs / nbz);
		var mod = nbs % nbz;
		var i = 0;		
		
		for(var z = 0; z < nbz; z++){
			for(var r = 0; r < res; r++){ 
				this._insertService(z, i++, 0, true);
			}		
			if(mod>0){
				try {
					this._insertService(z, i++, 0, true);
				}
				catch (e) {
					console.error("Unable to insert service in grid container", e, this.getChildren());
				}
				mod--;
			}else if(res === 0){ break; }
		}
	},
	
	_organizeServicesManually : function (){
		//summary: Organize Services by column property of widget.
		var children = this.getChildren();
		for(var i = 0; i < children.length; i++){
			try{
				this._insertService(children[i].column - 1, i, 0, true);
			}catch(e){
				console.error("Unable to insert service in grid container", e, children[i]);
			}
		}
		
	},
	
	_insertService : function(/*Integer*/z, /*Integer*/p, /*Integer*/i, /*Boolean*/first){
		//summary: Insert a service in a specific column of the GridContainer widget.
		var zone = this.cell[z];
		var kidsZone = zone.childNodes.length;
		var service = this.getChildren()[(i ? i : 0)];
		
		if(typeof(p)=="undefined" || p > kidsZone){ p = kidsZone; }		
		var toto = dojo.place(service.domNode,zone, p);
		service.domNode.setAttribute("tabIndex", 0);
		if(!service.dragRestriction)
            dojo.addClass(service.domNode,"dojoDndItem");
		if (!service.domNode.getAttribute("dndType")) service.domNode.setAttribute("dndType",service.declaredClass);
		dojox.layout.dnd._setGcDndHandle(service,this.withHandles,this.handleClasses, first);
		if(this.hasResizableColumns){		
			if(service.onLoad){
				this.connect(service, "onLoad", "_placeGrips");
			}
			if(service.onExecError){
				this.connect(service, "onExecError", "_placeGrips");
			}
			if(service.onUnLoad){
				this.connect(service, "onUnLoad", "_placeGrips");
			}
		}
		this._placeGrips();
		return service.id; // String
	},
	
	// FIXME: API change, rename to addChild
	addService : function(/*Object*/service, /*Integer*/z, /*Integer*/p){
		// summary: Add a service (child widget) in a specific column of the GridContainer widget.
		// service: 
		//	widget to insert
		// z:
		//	zone number (column)
		// p:
		//	place in the zone (first = 0)
		service.domNode.id = service.id;
		this.addChild(service);
		if(p <= 0){ p = 0; }
		var result = this._insertService(z,p);
		this.grid[z].setItem(service.id,{data: service.domNode, type: [service.domNode.getAttribute("dndType")]});
		return result; //Object
	},
	
	/***********grid methods******************/
	_createGrid : function(){
		//summary:  Create all grid (zones and grip)
		var grid = [];
		var i = 0;
		this.tabDZ = [];
		while(i < this.nbZones){
			var zone =	this.cell[i];
			this.tabDZ[i] = this._createZone(zone);
			if(this.hasResizableColumns && i != (this.nbZones-1)) {
				this._createGrip(this.tabDZ[i]);
			}
			grid.push(this.tabDZ[i]);
			i++;
		}
		if(this.hasResizableColumns){
			this.handleDndStart = [];
			for (var j = 0; j < this.tabDZ.length; j++) {
				var dz = this.tabDZ[j];
				var self = this;
				this.handleDndStart.push(dojo.connect(dz, "onDndStart", dz, function(source){
					if(source==this){
						self.handleDndInsertNodes = [];
						for (i = 0; i < self.tabDZ.length; i++) {
							self.handleDndInsertNodes.push(dojo.connect(self.tabDZ[i], "insertNodes", self, function(){
								self._disconnectDnd();
							}));
						}
						self.handleDndInsertNodes.push(dojo.connect(dz,"onDndCancel", self, self._disconnectDnd));
						self.onResized();
					}
				}));
			}
		}
		return grid; // Object
	},
	
	_disconnectDnd: function(){
		//summary: disconnect all events on insertNodes

		dojo.forEach(this.handleDndInsertNodes, dojo.disconnect);
		setTimeout(dojo.hitch(this, "onResized"), 0);
	},	
	
	_createZone: function(/*Object*/zone){
		//summary: Create a DnD column.
		var dz = null;
		dz = new dojox.layout.dnd.PlottedDnd(zone.id, {
			accept:this.acceptTypes,
			withHandles:this.withHandles,
			handleClasses: this.handleClasses, 
			singular: true, 
			hideSource:true,
			opacity: this.opacity,
			dom: this.domNode, 
			allowAutoScroll: this.allowAutoScroll,
			isOffset:this.isOffset,
			offsetDrag : this.offsetDrag
		});
		this.connect(dz, "insertDashedZone", "_placeGrips");
		this.connect(dz, "deleteDashedZone", "_placeGrips");
		return dz; //plottedDnd Object
	},

	/************  grips methods***************/

	_createGrip: function(/*Object*/dz){
		// summary: Create a grip for a specific zone
		var grip = document.createElement("div");
		grip.className = "gridContainerGrip";
		grip.setAttribute("tabIndex", "0");

		var _this= this;
		this.onMouseOver = this.connect(grip, "onmouseover", function(e){
				var gridContainerGripShow = false;
				for(var i = 0; i < _this.grid.length - 1; i++){
					if(dojo.hasClass(_this.grid[i].grip, "gridContainerGripShow")){
						gridContainerGripShow = true;
						break;
					}
				}
				if(!gridContainerGripShow){
					dojo.removeClass(e.target, "gridContainerGrip");
					dojo.addClass(e.target, "gridContainerGripShow");
				}
			}
		);

		this.connect(grip,"onmouseout",function(e){
				if(!_this._isResized){
					dojo.removeClass(e.target, "gridContainerGripShow");
					dojo.addClass(e.target, "gridContainerGrip");
				}
			}
		);

	 	this.connect(grip,"onmousedown",function(e){
			_this._a11yOn = false;
			_this._activeGrip = e.target;
			_this.resizeColumnOn(e);
		});
		
		this.domNode.appendChild(grip);
		dz.grip = grip;	
	},

	_initPlaceGrips: function(){
		//summary: Initialize the position of a grip which will not change (top)
		var dcs = dojo.getComputedStyle(this.domNode);
		var gcs = dojo.getComputedStyle(this.gridContainerTable);
		this._x = parseInt(dcs.paddingLeft);	
		this._topGrip = parseInt(dcs.paddingTop);
		if(dojo.isIE || gcs.borderCollapse != "collapse"){
			var ex = dojo._getBorderExtents(this.gridContainerTable);
			this._x += ex.l;
			this._topGrip += ex.t
		}
		this._topGrip += "px";

		dojo.forEach(this.grid, function(zone){
			if(zone.grip){
				var grip = zone.grip;
				if (!dojo.isIE){ 
					zone.pad = dojo._getPadBorderExtents(zone.node).w;
				}
				grip.style.top = this._topGrip;
			}
		}, this);
	},
	
	_placeGrips: function(){
		//summary: Define the position of a grip and place it on page.
		console.log('placegrips');
		var height;
		if (this.allowAutoScroll){
			height = this.gridNode.scrollHeight;
		}else{
			height = dojo.contentBox(this.gridNode).h;
		}
		var size = this._x;
		
		dojo.forEach(this.grid, function(zone){
			if (zone.grip){
				var grip = zone.grip;
				// Bug margin : IE 
				size += dojo[(dojo.isIE ? "marginBox" : "contentBox")](zone.node).w + (dojo.isIE ? 0 : zone.pad);
				dojo.style(grip,{
					left: size + "px",
					height: height + "px"
				});	
			}
		}, this);
	},
	
	_getZoneByIndex : function(/*Integer*/n){
		//summary: Return a DOM node containing a zone by given a index.
		return this.grid[(n >= 0 && n < this.grid.length ? n : 0 )]; //number
	},
	
	getIndexZone : function(/*Node*/zone){
		//summary: Return an integer by given a zone
		for(var z = 0; z < this.grid.length; z++){
			if(this.grid[z].domNode == zone){ 
				return z; // number 
			}
		}
		return -1; // number
	},
	
	/***********miscellaneous methods******************/	
	resizeColumnOn : function(/*Event*/e){
		// summary: Connect events to listen the resize action.
		//		Change the type of width columns (% to px) 
		//		Calculate the minwidth according to the children	
		var k = dojo.keys;
		if(this._a11yOn && e.keyCode != k.LEFT_ARROW && e.keyCode != k.RIGHT_ARROW){
			return;
		}
		e.preventDefault();		
		dojo.body().style.cursor = "ew-resize";
		this._isResized = true;		
		this.initX = e.pageX;
		var tabSize = [];
		for(var i = 0; i < this.grid.length; i++){
			tabSize[i] = dojo.contentBox(this.grid[i].node).w;
		}
		this.oldTabSize = tabSize;
		
		for(var i = 0; i< this.grid.length; i++){
			if(this._activeGrip == this.grid[i].grip) {
				this.currentColumn = this.grid[i].node;
				this.currentColumnWidth = tabSize[i];
				this.nextColumn = this.currentColumn.nextSibling;
				this.nextColumnWidth = tabSize[i+1];
			}
			this.grid[i].node.style.width = tabSize[i] + "px";
		}
		
		// calculate the minWidh of all children for current and next column
		var calculateChildMinWidth = function(childNodes, minChild){
			var width = 0;
			var childMinWidth = 0;
			dojo.forEach(childNodes, function(child){
				if(child.nodeType == 1){
					var objectStyle = dojo.getComputedStyle(child);
					var minWidth = (dojo.isIE ? minChild : parseInt(objectStyle.minWidth));

					childMinWidth = minWidth +
									parseInt(objectStyle.marginLeft)+
									parseInt(objectStyle.marginRight);
									
					if(width < childMinWidth){
						width = childMinWidth;
					}
				}
			});
			return width;
		};
		
		var currentColumnMinWidth = calculateChildMinWidth(this.currentColumn.childNodes, this.minChildWidth);
		var nextColumnMinWidth = calculateChildMinWidth(this.nextColumn.childNodes, this.minChildWidth);
		
		var minPix = Math.round((dojo.marginBox(this.gridContainerTable).w * this.minColWidth) / 100);
		this.currentMinCol = currentColumnMinWidth;
		this.nextMinCol = nextColumnMinWidth;

		if(minPix > this.currentMinCol){
			this.currentMinCol = minPix;
		}
		if(minPix > this.nextMinCol){
			this.nextMinCol = minPix;
		}
		if(this._a11yOn){
			this.connectResizeColumnMove = this.connect(dojo.doc, "onkeypress", "resizeColumnMove");
		}else{
			this.connectResizeColumnMove = this.connect(dojo.doc, "onmousemove", "resizeColumnMove");
			this.connectResizeColumnOff = this.connect(document, "onmouseup", "resizeColumnOff");
		}
		
	},
	
	resizeColumnMove: function(/*Event*/e){
		//summary: Change columns size.
		var d = 0;
		if(this._a11yOn){
			var k = dojo.keys;
			switch (e.keyCode){
				case k.LEFT_ARROW:
					d = -10;
					break;
				case k.RIGHT_ARROW:
					d = 10;
					break;
			}
		}else{
			e.preventDefault();
			d = e.pageX - this.initX;
		}
		if(d == 0){ return; }
		if(!(this.currentColumnWidth + d < this.currentMinCol || this.nextColumnWidth - d < this.nextMinCol)) {
			this.currentColumnWidth += d;
			this.nextColumnWidth -= d;
			this.initX = e.pageX;
			this.currentColumn.style["width"] = this.currentColumnWidth + "px";
			this.nextColumn.style["width"] = this.nextColumnWidth + "px";
			this._activeGrip.style.left = parseInt(this._activeGrip.style.left) + d + "px";
			this._placeGrips();
		}
		if(this._a11yOn){
			this.resizeColumnOff(e);
		}
	},
	
	resizeColumnOff : function(/*Event*/e){
		//summary: Disconnect resize events.
		//	Change the type of width columns (px to %)	
		dojo.body().style.cursor = "default";
		if(this._a11yOn){
			this.disconnect(this.connectResizeColumnMove);
			this._a11yOn = false;
		}else{
			this.disconnect(this.connectResizeColumnMove);
			this.disconnect(this.connectResizeColumnOff);
		}
		
		var tabSize = [];
		var testSize = [];
		var tabWidth = this.gridContainerTable.clientWidth;
		
		for(var i = 0; i < this.grid.length; i++){
			var _cb = dojo.contentBox(this.grid[i].node);
			if(dojo.isIE){
				tabSize[i] = dojo.marginBox(this.grid[i].node).w;
				testSize[i] = _cb.w;
			}else{
				tabSize[i] = _cb.w;
				testSize = tabSize;
			}
		}
		
		var update = false;
		for(var i = 0; i < testSize.length; i++){
			if(testSize[i] != this.oldTabSize[i]){
				update = true;
				break;
			}
		}
		if(update){
			var mul = dojo.isIE ? 100 : 10000;
			for(var i = 0; i < this.grid.length; i++){
				this.grid[i].node.style.width = Math.round((100 * mul * tabSize[i]) / tabWidth) / mul + "%";
			}
			this._placeGrips();
		}
		
		if (this._activeGrip){
			dojo.removeClass(this._activeGrip, "gridContainerGripShow");
			dojo.addClass(this._activeGrip, "gridContainerGrip");
		}
		this._isResized= false;
	},
	
	setColumns : function(/*Integer*/nbColumns){
		// summary: Set the number of columns
		if(nbColumns > 0){
			var delta = this.grid.length-nbColumns;
			if(delta > 0){ 
				var count = [];
				var zone, start, end;
				/*Check if right or left columns are fixed*/
				/*Columns are not taken in account and can't be deleted*/
				if(this.mode == "right"){
					end = (this.isLeftFixed && this.grid.length > 0) ? 1 : 0;
					start = this.grid.length - (this.isRightFixed ? 2 : 1);
					for(var z = start; z >= end; z--){
						var nbChildren = 0;
						var zone = this.grid[z].node;
						for(var j = 0;j < zone.childNodes.length; j++){
							if(zone.childNodes[j].nodeType==1 && !(zone.childNodes[j].id == "")){ //1 = dojo.html.ELEMENT_NODE
								nbChildren++;
								break;
							}
						}
						if(nbChildren == 0){
							count[count.length] = z;
						}
						if(count.length>=delta){
							this._deleteColumn(count);	
							break;	
						}
					}
					if(count.length < delta){
						//Not enough empty columns
						console.error(this.i18n.err_onSetNbColsRightMode);
					}
				}else{ // mode="left"
					if(this.isLeftFixed&&this.grid.length>0){
						start=1;
					}else{
						start=0;
					}
					if(this.isRightFixed){
						end=this.grid.length-1;
					}else{
						end=this.grid.length;
					}
					for(var z=start;z<end;z++){
						var nbChildren = 0;
						var zone = this.grid[z].node;
						for(var j = 0;j < zone.childNodes.length;j++){
							if(zone.childNodes[j].nodeType==1 && !(zone.childNodes[j].id == "")){ //1 = dojo.html.ELEMENT_NODE
								nbChildren++;
								break;
							}
						}
						if(nbChildren == 0){
							count[count.length] = z;
						} 
						if(count.length>=delta){
							this._deleteColumn(count);
							break;
						}
					}
					
					if (count.length<delta){
						//Not enough empty columns
						alert(this.i18n.err_onSetNbColsLeftMode);  
					}
				}
			}else{
				if(delta<0){ this._addColumn(Math.abs(delta)); }
			}
			this._initPlaceGrips();
			this._placeGrips();
		}	
	},
	
	_addColumn: function(/*Integer*/nbColumns){
		//summary: Add some columns	
		var node;
		//Add a grip to the last column
		if(this.hasResizableColumns && !this.isRightFixed && this.mode == "right"){
			node = this.grid[this.grid.length-1];			
			this._createGrip(node);			
		}
		
		for(i = 0;i < nbColumns; i++){
			node = dojo.doc.createElement("td");
			dojo.addClass(node,"gridContainerZone");
			//to fix IE Bug Border with empty cells
			node.id = this.id + "_dz" + this.nbZones;
			var dz;
			//MODIF MYS
			if(this.mode == "right"){
				if(this.isRightFixed){
					this.grid[this.grid.length-1].node.parentNode.insertBefore(node,this.grid[this.grid.length-1].node);
					dz = this._createZone(node);
					this.tabDZ.splice(this.tabDZ.length-1,0,dz);
					this.grid.splice(this.grid.length-1,0,dz);
					this.cell.splice(this.cell.length-1,0,node); 
				}else{
					var zone = this.gridNode.appendChild(node);
					dz = this._createZone(node);
					this.tabDZ.push(dz);
					this.grid.push(dz);
					this.cell.push(node); 
				}
			}else{
				if(this.isLeftFixed){
					(this.grid.length == 1) ? this.grid[0].node.parentNode.appendChild(node,this.grid[0].node) : this.grid[1].node.parentNode.insertBefore(node,this.grid[1].node);
					dz = this._createZone(node);
					this.tabDZ.splice(1,0,dz);
					this.grid.splice(1,0,dz);
					this.cell.splice(1,0,node); 
				}else{
					this.grid[this.grid.length-this.nbZones].node.parentNode.insertBefore(node,this.grid[this.grid.length-this.nbZones].node);
					dz = this._createZone(node);
					this.tabDZ.splice(this.tabDZ.length-this.nbZones,0,dz);
					this.grid.splice(this.grid.length-this.nbZones,0,dz);
					this.cell.splice(this.cell.length-this.nbZones,0,node);
				}
			}
			if(this.hasResizableColumns){
				// add a OnDndStart connect for each added columns
				var self = this;
				var handle = dojo.connect(dz, "onDndStart", dz, function(source){
					if(source == this){
						self.handleDndInsertNodes = [];
						for(var o = 0; o < self.tabDZ.length; o++){
							self.handleDndInsertNodes.push(dojo.connect(self.tabDZ[o], "insertNodes", self, function(){
								self._disconnectDnd();
							}));
						}
						self.handleDndInsertNodes.push(dojo.connect(dz, "onDndCancel", self, self._disconnectDnd));
						self.onResized();
					}
				});
				if(this.mode == "right"){
					  if(this.isRightFixed){
					  	this.handleDndStart.splice(this.handleDndStart.length - 1, 0, handle);
					  }else{
					  	this.handleDndStart.push(handle);
						}				
				}else{
					if (this.isLeftFixed){
						this.handleDndStart.splice(1, 0, handle);
					}else{
						this.handleDndStart.splice(this.handleDndStart.length - this.nbZones, 0, handle); 	
					}
				}
				//Add a grip to resize columns
				this._createGrip(dz);	
			}
			this.nbZones++;
		}
		this._updateColumnsWidth();
	},
	
	_deleteColumn: function(/*Array*/indices){
		//summary: Remove some columns with indices passed as an array
		var zone, child, nbDelZones;
		nbDelZones = 0;
		for(var i = 0; i < indices.length; i++){
			var idx = indices[i];
			if(this.mode == "right"){
				zone = this.grid[idx];
			}else{
				zone = this.grid[idx - nbDelZones];
			}
			for(var j = 0; j < zone.node.childNodes.length; j++){
				if(zone.node.childNodes[j].nodeType != 1){ continue; } //1 = dojo.html.ELEMENT_NODE
				child = dijit.byId(zone.node.childNodes[j].id);
				for(var x = 0; x < this.getChildren().length; x++){
					if(this.getChildren()[x] === child){
						this.getChildren().splice(x, 1);
						break;
					}
				}
			}
			zone.node.parentNode.removeChild(zone.node);
			if(this.mode == "right"){
				if(this.hasResizableColumns){
					dojo.disconnect(this.handleDndStart[idx]);
				}
				this.grid.splice(idx, 1);
				this.tabDZ.splice(idx, 1);
				this.cell.splice(idx, 1);
			}else{
				if(this.hasResizableColumns){
					dojo.disconnect(this.handleDndStart[idx - nbDelZones]);
				}
				this.grid.splice(idx - nbDelZones, 1);
				this.tabDZ.splice(idx - nbDelZones, 1);
				this.cell.splice(idx - nbDelZones, 1);
			}
			this.nbZones--;
			nbDelZones++;
			if(zone.grip){
				this.domNode.removeChild(zone.grip);
			}
		}
		this._updateColumnsWidth();
	},
	
	_updateColumnsWidth: function(){
		//summary: Update the columns width.
		var wCol = 100 / this.nbZones;
		var zone;
		for(var z = 0; z < this.grid.length; z++){
			zone = this.grid[z].node;
			zone.style.width = wCol + "%";
	 	}		
	},
	
	_selectFocus: function(/*Event*/event){
		//summary: 	Enable a11y into the GridContainer :
		//		- Possibility to move focus into the GridContainer (TAB, LEFT ARROW, RIGHT ARROW, UP ARROW, DOWN ARROW).
	    //		- Possibility to move GridContainer's children (Drag and Drop) with keyboard. (SHIFT +  ARROW). 
	    //		If the type of widget is not draggable, a popup is displayed. 
		var e = event.keyCode;
		var zone = null;
		var focus = dijit.getFocus();
		var focusNode = focus.node;
		var k = dojo.keys;
		var child = (e== k.UP_ARROW || e== k.LEFT_ARROW) ? "lastChild" : "firstChild";
		var pos = (e== k.UP_ARROW || e== k.LEFT_ARROW) ? "previousSibling" : "nextSibling";
		if (focusNode == this.containerNode) {
			switch (e) {
				case k.DOWN_ARROW:
				case k.RIGHT_ARROW:
					for(var i = 0; i < this.gridNode.childNodes.length; i++){
						zone = this.gridNode.childNodes[i].firstChild;
						var found = false;
						while(!found){
							if(zone != null){
								if(zone.style.display !== "none"){
									dijit.focus(zone);
									dojo.stopEvent(event);
									found = true;
								}else{
									zone = zone[pos];
								}	
							}else{ break; }
						}
						if(found){ break; }
					}
					break;
				case k.UP_ARROW:
				case k.LEFT_ARROW:
					for(var i = this.gridNode.childNodes.length - 1; i >= 0; i--){
						zone = this.gridNode.childNodes[i].lastChild;
						var found = false;
						while(!found){
							if(zone != null){
								if(zone.style.display !== "none"){
									dijit.focus(zone);
									dojo.stopEvent(event);
									found = true;
								}else{
									zone = zone[pos];
								}
							}else{ break; }
						}
						if(found){ break; }
					}
					break;
			}
		}else{
			if(focusNode.parentNode.parentNode == this.gridNode){
				switch(e){
					case k.UP_ARROW:
					case k.DOWN_ARROW:
						dojo.stopEvent(event);
						var nbDisplayChild = 0;
						dojo.forEach(focusNode.parentNode.childNodes, function(child){
							if (child.style.display !== "none")
								nbDisplayChild++;
						});
						if (nbDisplayChild == 1) return;
						var found = false;
						zone = focusNode[pos];				
						while(!found){
							if(zone == null){
								zone = focusNode.parentNode[child];
								if(zone.style.display !== "none") 
									found = true;
								else 
									zone = zone[pos];
							}else{
								if(zone.style.display !== "none"){ 
									found = true;
								}else{ 
									zone = zone[pos];
								}
							}
						}
						if(event.shiftKey){
							if (dijit.byNode(focusNode).dragRestriction)
								return;
							var _dndType = focusNode.getAttribute("dndtype");
							var accept = false;
							for(var i = 0; i < this.acceptTypes.length; i++){
								if (_dndType == this.acceptTypes[i]){
									var accept = true;
									break;
								}
							}
							if(accept){
								var parent = focusNode.parentNode;
								var firstChild = parent.firstChild;
								var lastChild = parent.lastChild;
								while(firstChild.style.display == "none" || lastChild.style.display == "none"){
									if(firstChild.style.display == "none"){
										firstChild = firstChild.nextSibling;
									}
									if(lastChild.style.display == "none"){
										lastChild = lastChild.previousSibling;
									}
								}
								if(e == k.UP_ARROW){
									var r = parent.removeChild(focusNode);
									if(r == firstChild){ 
										parent.appendChild(r);
									}else{ 
										parent.insertBefore(r, zone);
									}
									r.setAttribute("tabIndex", "0");
									dijit.focus(r);
								}else{
									if(focusNode == lastChild){
										var r = parent.removeChild(focusNode);
										parent.insertBefore(r, zone);
										r.setAttribute("tabIndex", "0");
										dijit.focus(r);
									}else{ 
										var r = parent.removeChild(zone);
										parent.insertBefore(r, focusNode);
										focusNode.setAttribute("tabIndex", "0");
										dijit.focus(focusNode);
									}
								}
							}else{
								this._displayPopup();	
							}
						}else{
							dijit.focus(zone);
						}
						break;
					case k.RIGHT_ARROW:
					case k.LEFT_ARROW:
						dojo.stopEvent(event);
						if(event.shiftKey){
							if(dijit.byNode(focusNode).dragRestriction){ return; }
							var z = 0;
							if(focusNode.parentNode[pos] == null){
								if (e == k.LEFT_ARROW){ var z = this.gridNode.childNodes.length - 1; }
							}else if(focusNode.parentNode[pos].nodeType == 3){
								z = this.gridNode.childNodes.length - 2;
							}else{
								for(var i = 0; i < this.gridNode.childNodes.length; i++){
									if(focusNode.parentNode[pos] == this.gridNode.childNodes[i]){ break; }
									z++;
								} 
							}
							var _dndType = focusNode.getAttribute("dndtype");
							var accept = false;
							for(var i = 0; i < this.acceptTypes.length; i++){
								if(_dndType == this.acceptTypes[i]){
									accept = true;
									break;
								}
							}
							if(accept){
								var parentSource = focusNode.parentNode;
								var widget = dijit.byNode(focusNode);
								var r = parentSource.removeChild(focusNode);
								var place = (e == k.RIGHT_ARROW ? 0 : this.gridNode.childNodes[z].length);
								this.addService(widget, z, place);
								r.setAttribute("tabIndex", "0");
								dijit.focus(r);
								this._placeGrips();
							}else{
								this._displayPopup();
							}
							
						}else{
							var node = focusNode.parentNode;
							
							while(zone === null){
								if(node[pos] !== null && node[pos].nodeType !== 3){
									node = node[pos];
								}else{
									if(pos === "previousSibling"){
										node = node.parentNode.childNodes[node.parentNode.childNodes.length - 1];
									}else{ 
										node = node.parentNode.childNodes[0];
									}	
								}
								var found = false;
								var tempZone = node[child];
								while(!found){
									if(tempZone != null){
										if(tempZone.style.display !== "none") {
											zone = tempZone;
											found = true;
										}else{
											tempZone = tempZone[pos];
										}
									}else{ break; }
								}
							}
							dijit.focus(zone);
						}
					break;
				}
			}else{
				// focus on a grip !
				if(dojo.hasClass(focusNode,"gridContainerGrip") || dojo.hasClass(focusNode,"gridContainerGripShow")){
					this._activeGrip = event.target;
					this._a11yOn = true; 
					this.resizeColumnOn(event);
				}
			}
		}
	},

	_displayPopup: function(){
		//summary: display a popup when a widget type can not move 	
		if(this._canDisplayPopup){
			var popup = dojo.doc.createElement("div");
			dojo.addClass(popup, "gridContainerPopup");
			popup.innerHTML = this.i18n.alertPopup;
			var attachPopup = this.containerNode.appendChild(popup);
			this._canDisplayPopup = false;
			setTimeout(dojo.hitch(this, function(){
				this.containerNode.removeChild(attachPopup);
				dojo._destroyElement(attachPopup);
				this._canDisplayPopup = true;
			}), this.timeDisplayPopup);
		}
	}
	
});

dojo.extend(dijit._Widget, {
	// dragRestriction: Boolean
	//		To remove the drag capability.
	dragRestriction : false,

	// column: String
	//		Column of the grid to place the widget.
	column : "1",

	// group: String
	//		Defines a group belonging.
	group : ""
	
});