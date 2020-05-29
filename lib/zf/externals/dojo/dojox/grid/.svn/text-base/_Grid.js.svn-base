dojo.provide("dojox.grid._Grid");

dojo.require("dojox.html.metrics");
dojo.require("dojox.grid.util");
dojo.require("dojox.grid._Scroller");
dojo.require("dojox.grid._Layout");
dojo.require("dojox.grid._View");
dojo.require("dojox.grid._ViewManager");
dojo.require("dojox.grid._RowManager");
dojo.require("dojox.grid._FocusManager");
dojo.require("dojox.grid._EditManager");
dojo.require("dojox.grid.Selection");
dojo.require("dojox.grid._RowSelector");
dojo.require("dojox.grid._Events");

dojo.require("dijit.Menu");

dojo.requireLocalization("dijit", "loading");

(function(){
	var jobs = {
		cancel: function(inHandle){
			if(inHandle){
				clearTimeout(inHandle);
			}
		},

		jobs: [],

		job: function(inName, inDelay, inJob){
			jobs.cancelJob(inName);
			var job = function(){

				delete jobs.jobs[inName];
				inJob();
			}
			jobs.jobs[inName] = setTimeout(job, inDelay);
		},

		cancelJob: function(inName){
			jobs.cancel(jobs.jobs[inName]);
		}
	};

	dojo.declare('dojox.grid._Grid',
		[ dijit._Widget, dijit._Templated, dojox.grid._Events ],
		{
		// summary:
		// 		A grid widget with virtual scrolling, cell editing, complex rows,
		// 		sorting, fixed columns, sizeable columns, etc.
		//
		//	description:
		//		_Grid provides the full set of grid features without any
		//		direct connection to a data store.
		//
		//		The grid exposes a get function for the grid, or optionally
		//		individual columns, to populate cell contents.
		//
		//		The grid is rendered based on its structure, an object describing
		//		column and cell layout.
		//
		//	example:
		//		A quick sample:
		//
		//		define a get function
		//	|	function get(inRowIndex){ // called in cell context
		//	|		return [this.index, inRowIndex].join(', ');
		//	|	}
		//
		//		define the grid structure:
		//	|	var structure = [ // array of view objects
		//	|		{ cells: [// array of rows, a row is an array of cells
		//	|			[
		//	|				{ name: "Alpha", width: 6 },
		//	|				{ name: "Beta" },
		//	|				{ name: "Gamma", get: get }]
		//	|		]}
		//	|	];
		//
		//	|	<div id="grid"
		//	|		rowCount="100" get="get"
		//	|		structure="structure"
		//	|		dojoType="dojox.grid._Grid"></div>

		templatePath: dojo.moduleUrl("dojox.grid","resources/_Grid.html"),

		// classTag: String
		// 		CSS class applied to the grid's domNode
		classTag: 'dojoxGrid',

		get: function(inRowIndex){
			// summary: Default data getter.
			// description:
			//		Provides data to display in a grid cell. Called in grid cell context.
			//		So this.cell.index is the column index.
			// inRowIndex: Integer
			//		Row for which to provide data
			// returns:
			//		Data to display for a given grid cell.
		},

		// settings
		// rowCount: Integer
		//		Number of rows to display.
		rowCount: 5,

		// keepRows: Integer
		//		Number of rows to keep in the rendering cache.
		keepRows: 75,

		// rowsPerPage: Integer
		//		Number of rows to render at a time.
		rowsPerPage: 25,

		// autoWidth: Boolean
		//		If autoWidth is true, grid width is automatically set to fit the data.
		autoWidth: false,

		// autoHeight: Boolean|Integer
		//		If autoHeight is true, grid height is automatically set to fit the data.
		//		If it is an integer, the height will be automatically set to fit the data
		//		if there are fewer than that many rows - and the height will be set to show
		//		that many rows if there are more
		autoHeight: '',

		// autoRender: Boolean
		//		If autoRender is true, grid will render itself after initialization.
		autoRender: true,

		// defaultHeight: String
		//		default height of the grid, measured in any valid css unit.
		defaultHeight: '15em',
		
		// height: String
		//		explicit height of the grid, measured in any valid css unit.  This will be populated (and overridden)
		//		if the height: css attribute exists on the source node.
		height: '',

		// structure: Object|String
		//		View layout defintion. Can be set to a layout object, or to the (string) name of a layout object.
		structure: '',

		// elasticView: Integer
		//	Override defaults and make the indexed grid view elastic, thus filling available horizontal space.
		elasticView: -1,

		// singleClickEdit: boolean
		//		Single-click starts editing. Default is double-click
		singleClickEdit: false,

		// selectionMode: String
		//		Set the selection mode of grid's Selection.  Value must be 'single', 'multiple',
		//		or 'extended'.  Default is 'extended'.
		selectionMode: 'extended',

		// rowSelector: Boolean|String
		// 		If set to true, will add a row selector view to this grid.  If set to a CSS width, will add
		// 		a row selector of that width to this grid.
		rowSelector: '',

		// columnReordering: Boolean
		// 		If set to true, will add drag and drop reordering to views with one row of columns.
		columnReordering: false,

		// headerMenu: dijit.Menu
		// 		If set to a dijit.Menu, will use this as a context menu for the grid headers.
		headerMenu: null,

		// placeholderLabel: String
		// 		Label of placeholders to search for in the header menu to replace with column toggling
		// 		menu items.
		placeholderLabel: "GridColumns",
		
		// Used to store the last two clicks, to ensure double-clicking occurs based on the intended row
		_click: null,
		
		// loadingMessage: String
		//  Message that shows while the grid is loading
		loadingMessage: "<span class='dojoxGridLoading'>${loadingState}</span>",

		// errorMessage: String
		//  Message that shows when the grid encounters an error loading
		errorMessage: "<span class='dojoxGridError'>${errorState}</span>",

		// noDataMessage: String
		//  Message that shows if the grid has no data - wrap it in a
		//  span with class 'dojoxGridNoData' if you want it to be
		//  styled similar to the loading and error messages
		noDataMessage: "",

		// private
		sortInfo: 0,
		themeable: true,
		_placeholders: null,

		// initialization
		buildRendering: function(){
			this.inherited(arguments);
			// reset get from blank function (needed for markup parsing) to null, if not changed
			if(this.get == dojox.grid._Grid.prototype.get){
				this.get = null;
			}
			if(!this.domNode.getAttribute('tabIndex')){
				this.domNode.tabIndex = "0";
			}
			this.createScroller();
			this.createLayout();
			this.createViews();
			this.createManagers();

			this.createSelection();

			this.connect(this.selection, "onSelected", "onSelected");
			this.connect(this.selection, "onDeselected", "onDeselected");
			this.connect(this.selection, "onChanged", "onSelectionChanged");

			dojox.html.metrics.initOnFontResize();
			this.connect(dojox.html.metrics, "onFontResize", "textSizeChanged");
			dojox.grid.util.funnelEvents(this.domNode, this, 'doKeyEvent', dojox.grid.util.keyEvents);
			this.connect(this, "onShow", "renderOnIdle");
		},
		
		postMixInProperties: function(){
			this.inherited(arguments);
			var messages = dojo.i18n.getLocalization("dijit", "loading", this.lang);
			this.loadingMessage = dojo.string.substitute(this.loadingMessage, messages);
			this.errorMessage = dojo.string.substitute(this.errorMessage, messages);
			if(this.srcNodeRef && this.srcNodeRef.style.height){
				this.height = this.srcNodeRef.style.height;
			}
			// Call this to update our autoheight to start out
			this._setAutoHeightAttr(this.autoHeight, true);
		},
		
		postCreate: function(){
			// replace stock styleChanged with one that triggers an update
			this.styleChanged = this._styleChanged;
			this._placeholders = [];
			this.setHeaderMenu(this.headerMenu);
			this.setStructure(this.structure);
			this._click = [];
		},

		destroy: function(){
			this.domNode.onReveal = null;
			this.domNode.onSizeChange = null;

			this.edit.destroy();
			delete this.edit;

			this.views.destroyViews();
			if(this.scroller){
				this.scroller.destroy();
				delete this.scroller;
			}
			if(this.focus){
				this.focus.destroy();
				delete this.focus;
			}
			if(this.headerMenu&&this._placeholders.length){
				dojo.forEach(this._placeholders, function(p){ p.unReplace(true); });
				this.headerMenu.unBindDomNode(this.viewsHeaderNode);
			}
			this.inherited(arguments);
		},

		_setAutoHeightAttr: function(ah, skipRender){
			// Calculate our autoheight - turn it into a boolean or an integer
			if(typeof ah == "string"){
				if(!ah || ah == "false"){
					ah = false;
				}else if (ah == "true"){
					ah = true;
				}else{
					ah = window.parseInt(ah, 10);
					if(isNaN(ah)){
						ah = false;
					}
					// Autoheight must be at least 1, if it's a number.  If it's
					// less than 0, we'll take that to mean "all" rows (same as 
					// autoHeight=true - if it is equal to zero, we'll take that
					// to mean autoHeight=false
					if(ah < 0){
						ah = true;
					}else if (ah === 0){
						ah = false;
					}
				}
			}
			this.autoHeight = ah;
			if(typeof ah == "boolean"){
				this._autoHeight = ah;
			}else if(typeof ah == "number"){
				this._autoHeight = (ah >= this.rowCount);
			}else{
				this._autoHeight = false;
			}
			if(this._started && !skipRender){
				this.render();
			}
		},
		
		styleChanged: function(){
			this.setStyledClass(this.domNode, '');
		},

		_styleChanged: function(){
			this.styleChanged();
			this.update();
		},

		textSizeChanged: function(){
			setTimeout(dojo.hitch(this, "_textSizeChanged"), 1);
		},

		_textSizeChanged: function(){
			if(this.domNode){
				this.views.forEach(function(v){
					v.content.update();
				});
				this.render();
			}
		},

		sizeChange: function(){
			jobs.job(this.id + 'SizeChange', 50, dojo.hitch(this, "update"));
		},

		renderOnIdle: function() {
			setTimeout(dojo.hitch(this, "render"), 1);
		},

		createManagers: function(){
			// summary:
			//		create grid managers for various tasks including rows, focus, selection, editing

			// row manager
			this.rows = new dojox.grid._RowManager(this);
			// focus manager
			this.focus = new dojox.grid._FocusManager(this);
			// edit manager
			this.edit = new dojox.grid._EditManager(this);
		},

		createSelection: function(){
			// selection manager
			this.selection = new dojox.grid.Selection(this);
		},

		createScroller: function(){
			// summary: Creates a new virtual scroller
			this.scroller = new dojox.grid._Scroller();
			this.scroller.grid = this;
			this.scroller._pageIdPrefix = this.id + '-';
			this.scroller.renderRow = dojo.hitch(this, "renderRow");
			this.scroller.removeRow = dojo.hitch(this, "rowRemoved");
		},

		createLayout: function(){
			// summary: Creates a new Grid layout
			this.layout = new dojox.grid._Layout(this);
			this.connect(this.layout, "moveColumn", "onMoveColumn");
		},

		onMoveColumn: function(){
			this.render();
			this._resize();
		},

		// views
		createViews: function(){
			this.views = new dojox.grid._ViewManager(this);
			this.views.createView = dojo.hitch(this, "createView");
		},

		createView: function(inClass, idx){
			var c = dojo.getObject(inClass);
			var view = new c({ grid: this, index: idx });
			this.viewsNode.appendChild(view.domNode);
			this.viewsHeaderNode.appendChild(view.headerNode);
			this.views.addView(view);
			return view;
		},

		buildViews: function(){
			for(var i=0, vs; (vs=this.layout.structure[i]); i++){
				this.createView(vs.type || dojox._scopeName + ".grid._View", i).setStructure(vs);
			}
			this.scroller.setContentNodes(this.views.getContentNodes());
		},

		setStructure: function(inStructure){
			// summary:
			//		Install a new structure and rebuild the grid.
			// inStructure: Object
			//		Structure object defines the grid layout and provides various
			//		options for grid views and columns
			//	description:
			//		A grid structure is an array of view objects. A view object can
			//		specify a view type (view class), width, noscroll (boolean flag
			//		for view scrolling), and cells. Cells is an array of objects
			//		corresponding to each grid column. The view cells object is an
			//		array of subrows comprising a single row. Each subrow is an
			//		array of column objects. A column object can have a name,
			//		width, value (default), get function to provide data, styles,
			//		and span attributes (rowSpan, colSpan).

			var s = inStructure;
			if(s && dojo.isString(s)){
				s=dojo.getObject(s);
			}
			if(!s){
				if(this.layout.structure){
					s = this.layout.structure;
				}else{
					return;
				}
			}
			this.views.destroyViews();
			if(s !== this.layout.structure){
				this.layout.setStructure(s);
			}
			this._structureChanged();
		},
		
		getColumnTogglingItems: function(){
			// Summary: returns an array of dijit.CheckedMenuItem widgets that can be
			//		added to a menu for toggling columns on and off.
			return dojo.map(this.layout.cells, function(cell){
				if(!cell.menuItems){ cell.menuItems = []; }

				var self = this;
				var item = new dijit.CheckedMenuItem({
					label: cell.name,
					checked: !cell.hidden,
					_gridCell: cell,
					onChange: function(checked){
						if(self.layout.setColumnVisibility(this._gridCell.index, checked)){
							var items = this._gridCell.menuItems;
							if(items.length > 1){
								dojo.forEach(items, function(item){
									if(item !== this){
										item.setAttribute("checked", checked);
									}
								}, this);
							}
							var checked = dojo.filter(self.layout.cells, function(c){
								if(c.menuItems.length > 1){
									dojo.forEach(c.menuItems, "item.attr('disabled', false);");
								}else{
									c.menuItems[0].attr('disabled', false);
								}
								return !c.hidden;
							});
							if(checked.length == 1){
								dojo.forEach(checked[0].menuItems, "item.attr('disabled', true);");
							}
						}
					},
					destroy: function(){
						var index = dojo.indexOf(this._gridCell.menuItems, this);
						this._gridCell.menuItems.splice(index, 1);
						delete this._gridCell;
						dijit.CheckedMenuItem.prototype.destroy.apply(this, arguments);
					}
				});
				cell.menuItems.push(item);
				return item;
			}, this); // dijit.CheckedMenuItem[]
		},

		setHeaderMenu: function(menu){
			if(this._placeholders.length){
				dojo.forEach(this._placeholders, function(p){
					p.unReplace(true);
				});
				this._placeholders = [];
			}
			if(this.headerMenu){
				this.headerMenu.unBindDomNode(this.viewsHeaderNode);
			}
			this.headerMenu = menu;
			if(!menu){ return; }

			this.headerMenu.bindDomNode(this.viewsHeaderNode);
			if(this.headerMenu.getPlaceholders){
				this._placeholders = this.headerMenu.getPlaceholders(this.placeholderLabel);
			}
		},
		
		setupHeaderMenu: function(){
			if(this._placeholders && this._placeholders.length){
				dojo.forEach(this._placeholders, function(p){
					if(p._replaced){
						p.unReplace(true);
					}
					p.replace(this.getColumnTogglingItems());
				}, this);
			}
		},

		_fetch: function(start){
			this.setScrollTop(0);
		},

		getItem: function(inRowIndex){
			return null;
		},
		
		showMessage: function(message){
			if(message){
				this.messagesNode.innerHTML = message;
				this.messagesNode.style.display = "";
			}else{
				this.messagesNode.innerHTML = "";
				this.messagesNode.style.display = "none";
			}
		},

		_structureChanged: function() {
			this.buildViews();
			if(this.autoRender && this._started){
				this.render();
			}
		},

		hasLayout: function() {
			return this.layout.cells.length;
		},

		// sizing
		resize: function(changeSize, resultSize){
			// summary:
			//		Update the grid's rendering dimensions and resize it
			// sizeBox: Object?
			//		{w: int, h: int, l: int, t: int}

			// see dijit.layout._LayoutWidget for details on resize argument meaning
			// TODO: even when Grid is a child of BorderContainer,
			// the current code in _resize() calls contentBox() to set the size,
			// which is inappropriate since BorderContainer has already done that.
			// Should pass changeSize and resultSize to _resize(), and only act on
			// changeSize
			var sizeBox = {};
			dojo.mixin(sizeBox, resultSize || {});
			dojo.mixin(sizeBox, changeSize || {});

			// FIXME: If grid is not sized explicitly, sometimes bogus scrollbars
			// can appear in our container, which may require an extra call to 'resize'
			// to sort out.
			this._sizeBox = sizeBox;
			this._resize();
			this.sizeChange();
		},

		_getPadBorder: function() {
			this._padBorder = this._padBorder || dojo._getPadBorderExtents(this.domNode);
			return this._padBorder;
		},

		_getHeaderHeight: function(){
			var vns = this.viewsHeaderNode.style, t = vns.display == "none" ? 0 : this.views.measureHeader();
			vns.height = t + 'px';
			// header heights are reset during measuring so must be normalized after measuring.
			this.views.normalizeHeaderNodeHeight();
			return t;
		},
		
		_resize: function(){
			// if we have set up everything except the DOM, we cannot resize
			var pn = this.domNode.parentNode;
			if(!pn || pn.nodeType != 1 || !this.hasLayout() || pn.style.visibility == "hidden" || pn.style.display == "none"){
				return;
			}
			// useful measurement
			var padBorder = this._getPadBorder();
			// grid height
			if(this._autoHeight){
				this.domNode.style.height = 'auto';
				this.viewsNode.style.height = '';
			}else if(typeof this.autoHeight == "number"){
				var h = this._getHeaderHeight();
				h += (this.scroller.averageRowHeight * this.autoHeight);
				this.domNode.style.height = h + "px";
			}else if(this.flex > 0){
			}else if(this.domNode.clientHeight <= padBorder.h){
				if(pn == document.body){
					this.domNode.style.height = this.defaultHeight;
				}else if(this.height){
					this.domNode.style.height = this.height;
				}else{
					this.fitTo = "parent";
				}
			}
			// if we are given dimensions, size the grid's domNode to those dimensions
			if(this._sizeBox){
				dojo.contentBox(this.domNode, this._sizeBox);	// TODO: should be marginBox() to match dijit behavior
			}else if(this.fitTo == "parent"){
				var h = dojo._getContentBox(pn).h;
				dojo.marginBox(this.domNode, { h: Math.max(0, h) });
			}

			var h = dojo._getContentBox(this.domNode).h;
			if(h == 0 && !this._autoHeight){
				// We need to hide the header, since the Grid is essentially hidden.
				this.viewsHeaderNode.style.display = "none";
			}else{
				// Otherwise, show the header and give it an appropriate height.
				this.viewsHeaderNode.style.display = "block";
				this._getHeaderHeight();
			}

			// NOTE: it is essential that width be applied before height
			// Header height can only be calculated properly after view widths have been set.
			// This is because flex column width is naturally 0 in Firefox.
			// Therefore prior to width sizing flex columns with spaces are maximally wrapped
			// and calculated to be too tall.
			this.adaptWidth();
			this.adaptHeight();

			this.postresize();
		},

		adaptWidth: function() {
			// private: sets width and position for views and update grid width if necessary
			var w = this.autoWidth ? 0 : this.domNode.clientWidth || (this.domNode.offsetWidth - this._getPadBorder().w),
				vw = this.views.arrange(1, w);
			this.views.onEach("adaptWidth");
			if (this.autoWidth)
				this.domNode.style.width = vw + "px";
		},

		adaptHeight: function(){
			// private: measures and normalizes header height, then sets view heights, and then updates scroller
			// content extent
			var t = this._getHeaderHeight();
			var h = (this._autoHeight ? -1 : Math.max(this.domNode.clientHeight - t, 0) || 0);
			this.views.onEach('setSize', [0, h]);
			this.views.onEach('adaptHeight');
			if(!this._autoHeight){
				var numScroll = 0, numNoScroll = 0;
				var noScrolls = dojo.filter(this.views.views, function(v){
					var has = v.hasHScrollbar();
					if(has){ numScroll++; }else{ numNoScroll++; }
					return (!has);
				});
				if(numScroll > 0 && numNoScroll > 0){
					dojo.forEach(noScrolls, function(v){
						v.adaptHeight(true);
					});
				}
			}
			if(this.autoHeight === true || h != -1 || (typeof this.autoHeight == "number" && this.autoHeight >= this.rowCount)){
				this.scroller.windowHeight = h;
			}else{
				this.scroller.windowHeight = Math.max(this.domNode.clientHeight - t, 0);
			}
		},

		// startup
		startup: function(){
			if(this._started){return;}
			
			this.inherited(arguments);

			this.render();
		},

		// render
		render: function(){
			// summary:
			//	Render the grid, headers, and views. Edit and scrolling states are reset. To retain edit and
			// scrolling states, see Update.

			if(!this.domNode){return;}
			if(!this._started){return;}

			if(!this.hasLayout()) {
				this.scroller.init(0, this.keepRows, this.rowsPerPage);
				return;
			}
			//
			this.update = this.defaultUpdate;
			this._render();
		},

		_render: function(){
			this.scroller.init(this.rowCount, this.keepRows, this.rowsPerPage);
			this.prerender();
			this.setScrollTop(0);
			this.postrender();
		},

		prerender: function(){
			// if autoHeight, make sure scroller knows not to virtualize; everything must be rendered.
			this.keepRows = this._autoHeight ? 0 : this.constructor.prototype.keepRows;
			this.scroller.setKeepInfo(this.keepRows);
			this.views.render();
			this._resize();
		},

		postrender: function(){
			this.postresize();
			this.focus.initFocusView();
			// make rows unselectable
			dojo.setSelectable(this.domNode, false);
		},

		postresize: function(){
			// views are position absolute, so they do not inflate the parent
			if(this._autoHeight){
				var size = Math.max(this.views.measureContent()) + 'px';
				this.viewsNode.style.height = size;
			}
		},

		renderRow: function(inRowIndex, inNodes){
			// summary: private, used internally to render rows
			this.views.renderRow(inRowIndex, inNodes);
		},

		rowRemoved: function(inRowIndex){
			// summary: private, used internally to remove rows
			this.views.rowRemoved(inRowIndex);
		},

		invalidated: null,

		updating: false,

		beginUpdate: function(){
			// summary:
			//		Use to make multiple changes to rows while queueing row updating.
			// NOTE: not currently supporting nested begin/endUpdate calls
			this.invalidated = [];
			this.updating = true;
		},

		endUpdate: function(){
			// summary:
			//		Use after calling beginUpdate to render any changes made to rows.
			this.updating = false;
			var i = this.invalidated, r;
			if(i.all){
				this.update();
			}else if(i.rowCount != undefined){
				this.updateRowCount(i.rowCount);
			}else{
				for(r in i){
					this.updateRow(Number(r));
				}
			}
			this.invalidated = null;
		},

		// update
		defaultUpdate: function(){
			// note: initial update calls render and subsequently this function.
			if(!this.domNode){return;}
			if(this.updating){
				this.invalidated.all = true;
				return;
			}
			//this.edit.saveState(inRowIndex);
			var lastScrollTop = this.scrollTop;
			this.prerender();
			this.scroller.invalidateNodes();
			this.setScrollTop(lastScrollTop);
			this.postrender();
			//this.edit.restoreState(inRowIndex);
		},

		update: function(){
			// summary:
			//		Update the grid, retaining edit and scrolling states.
			this.render();
		},

		updateRow: function(inRowIndex){
			// summary:
			//		Render a single row.
			// inRowIndex: Integer
			//		Index of the row to render
			inRowIndex = Number(inRowIndex);
			if(this.updating){
				this.invalidated[inRowIndex]=true;
			}else{
				this.views.updateRow(inRowIndex);
				this.scroller.rowHeightChanged(inRowIndex);
			}
		},

		updateRows: function(startIndex, howMany){
			// summary:
			//		Render consecutive rows at once.
			// startIndex: Integer
			//		Index of the starting row to render
			// howMany: Integer
			//		How many rows to update.
			startIndex = Number(startIndex);
			howMany = Number(howMany);
			if(this.updating){
				for(var i=0; i<howMany; i++){
					this.invalidated[i+startIndex]=true;
				}
			}else{
				for(var i=0; i<howMany; i++){
					this.views.updateRow(i+startIndex);
				}
				this.scroller.rowHeightChanged(startIndex);
			}
		},

		updateRowCount: function(inRowCount){
			//summary:
			//	Change the number of rows.
			// inRowCount: int
			//	Number of rows in the grid.
			if(this.updating){
				this.invalidated.rowCount = inRowCount;
			}else{
				this.rowCount = inRowCount;
				this._setAutoHeightAttr(this.autoHeight, true);
				if(this.layout.cells.length){
					this.scroller.updateRowCount(inRowCount);
				}
				this._resize();				
				if(this.layout.cells.length){
					this.setScrollTop(this.scrollTop);
				}
			}
		},

		updateRowStyles: function(inRowIndex){
			// summary:
			//		Update the styles for a row after it's state has changed.
			this.views.updateRowStyles(inRowIndex);
		},

		rowHeightChanged: function(inRowIndex){
			// summary:
			//		Update grid when the height of a row has changed. Row height is handled automatically as rows
			//		are rendered. Use this function only to update a row's height outside the normal rendering process.
			// inRowIndex: Integer
			// 		index of the row that has changed height

			this.views.renormalizeRow(inRowIndex);
			this.scroller.rowHeightChanged(inRowIndex);
		},

		// fastScroll: Boolean
		//		flag modifies vertical scrolling behavior. Defaults to true but set to false for slower
		//		scroll performance but more immediate scrolling feedback
		fastScroll: true,

		delayScroll: false,

		// scrollRedrawThreshold: int
		//	pixel distance a user must scroll vertically to trigger grid scrolling.
		scrollRedrawThreshold: (dojo.isIE ? 100 : 50),

		// scroll methods
		scrollTo: function(inTop){
			// summary:
			//		Vertically scroll the grid to a given pixel position
			// inTop: Integer
			//		vertical position of the grid in pixels
			if(!this.fastScroll){
				this.setScrollTop(inTop);
				return;
			}
			var delta = Math.abs(this.lastScrollTop - inTop);
			this.lastScrollTop = inTop;
			if(delta > this.scrollRedrawThreshold || this.delayScroll){
				this.delayScroll = true;
				this.scrollTop = inTop;
				this.views.setScrollTop(inTop);
				jobs.job('dojoxGridScroll', 200, dojo.hitch(this, "finishScrollJob"));
			}else{
				this.setScrollTop(inTop);
			}
		},

		finishScrollJob: function(){
			this.delayScroll = false;
			this.setScrollTop(this.scrollTop);
		},

		setScrollTop: function(inTop){
			this.scroller.scroll(this.views.setScrollTop(inTop));
		},

		scrollToRow: function(inRowIndex){
			// summary:
			//		Scroll the grid to a specific row.
			// inRowIndex: Integer
			// 		grid row index
			this.setScrollTop(this.scroller.findScrollTop(inRowIndex) + 1);
		},

		// styling (private, used internally to style individual parts of a row)
		styleRowNode: function(inRowIndex, inRowNode){
			if(inRowNode){
				this.rows.styleRowNode(inRowIndex, inRowNode);
			}
		},
		
		// called when the mouse leaves the grid so we can deselect all hover rows
		_mouseOut: function(e){
			this.rows.setOverRow(-2);
		},
	
		// cells
		getCell: function(inIndex){
			// summary:
			//		Retrieves the cell object for a given grid column.
			// inIndex: Integer
			// 		Grid column index of cell to retrieve
			// returns:
			//		a grid cell
			return this.layout.cells[inIndex];
		},

		setCellWidth: function(inIndex, inUnitWidth) {
			this.getCell(inIndex).unitWidth = inUnitWidth;
		},

		getCellName: function(inCell){
			// summary: Returns the cell name of a passed cell
			return "Cell " + inCell.index; // String
		},

		// sorting
		canSort: function(inSortInfo){
			// summary:
			//		Determines if the grid can be sorted
			// inSortInfo: Integer
			//		Sort information, 1-based index of column on which to sort, positive for an ascending sort
			// 		and negative for a descending sort
			// returns: Boolean
			//		True if grid can be sorted on the given column in the given direction
		},

		sort: function(){
		},

		getSortAsc: function(inSortInfo){
			// summary:
			//		Returns true if grid is sorted in an ascending direction.
			inSortInfo = inSortInfo == undefined ? this.sortInfo : inSortInfo;
			return Boolean(inSortInfo > 0); // Boolean
		},

		getSortIndex: function(inSortInfo){
			// summary:
			//		Returns the index of the column on which the grid is sorted
			inSortInfo = inSortInfo == undefined ? this.sortInfo : inSortInfo;
			return Math.abs(inSortInfo) - 1; // Integer
		},

		setSortIndex: function(inIndex, inAsc){
			// summary:
			// 		Sort the grid on a column in a specified direction
			// inIndex: Integer
			// 		Column index on which to sort.
			// inAsc: Boolean
			// 		If true, sort the grid in ascending order, otherwise in descending order
			var si = inIndex +1;
			if(inAsc != undefined){
				si *= (inAsc ? 1 : -1);
			} else if(this.getSortIndex() == inIndex){
				si = -this.sortInfo;
			}
			this.setSortInfo(si);
		},

		setSortInfo: function(inSortInfo){
			if(this.canSort(inSortInfo)){
				this.sortInfo = inSortInfo;
				this.sort();
				this.update();
			}
		},

		// DOM event handler
		doKeyEvent: function(e){
			e.dispatch = 'do' + e.type;
			this.onKeyEvent(e);
		},

		// event dispatch
		//: protected
		_dispatch: function(m, e){
			if(m in this){
				return this[m](e);
			}
		},

		dispatchKeyEvent: function(e){
			this._dispatch(e.dispatch, e);
		},

		dispatchContentEvent: function(e){
			this.edit.dispatchEvent(e) || e.sourceView.dispatchContentEvent(e) || this._dispatch(e.dispatch, e);
		},

		dispatchHeaderEvent: function(e){
			e.sourceView.dispatchHeaderEvent(e) || this._dispatch('doheader' + e.type, e);
		},

		dokeydown: function(e){
			this.onKeyDown(e);
		},

		doclick: function(e){
			if(e.cellNode){
				this.onCellClick(e);
			}else{
				this.onRowClick(e);
			}
		},

		dodblclick: function(e){
			if(e.cellNode){
				this.onCellDblClick(e);
			}else{
				this.onRowDblClick(e);
			}
		},

		docontextmenu: function(e){
			if(e.cellNode){
				this.onCellContextMenu(e);
			}else{
				this.onRowContextMenu(e);
			}
		},

		doheaderclick: function(e){
			if(e.cellNode){
				this.onHeaderCellClick(e);
			}else{
				this.onHeaderClick(e);
			}
		},

		doheaderdblclick: function(e){
			if(e.cellNode){
				this.onHeaderCellDblClick(e);
			}else{
				this.onHeaderDblClick(e);
			}
		},

		doheadercontextmenu: function(e){
			if(e.cellNode){
				this.onHeaderCellContextMenu(e);
			}else{
				this.onHeaderContextMenu(e);
			}
		},

		// override to modify editing process
		doStartEdit: function(inCell, inRowIndex){
			this.onStartEdit(inCell, inRowIndex);
		},

		doApplyCellEdit: function(inValue, inRowIndex, inFieldIndex){
			this.onApplyCellEdit(inValue, inRowIndex, inFieldIndex);
		},

		doCancelEdit: function(inRowIndex){
			this.onCancelEdit(inRowIndex);
		},

		doApplyEdit: function(inRowIndex){
			this.onApplyEdit(inRowIndex);
		},

		// row editing
		addRow: function(){
			// summary:
			//		Add a row to the grid.
			this.updateRowCount(this.rowCount+1);
		},

		removeSelectedRows: function(){
			// summary:
			//		Remove the selected rows from the grid.
			this.updateRowCount(Math.max(0, this.rowCount - this.selection.getSelected().length));
			this.selection.clear();
		}

	});

	dojox.grid._Grid.markupFactory = function(props, node, ctor, cellFunc){
		var d = dojo;
		var widthFromAttr = function(n){
			var w = d.attr(n, "width")||"auto";
			if((w != "auto")&&(w.slice(-2) != "em")&&(w.slice(-1) != "%")){
				w = parseInt(w)+"px";
			}
			return w;
		}
		// if(!props.store){ console.debug("no store!"); }
		// if a structure isn't referenced, do we have enough
		// data to try to build one automatically?
		if(	!props.structure &&
			node.nodeName.toLowerCase() == "table"){

			// try to discover a structure
			props.structure = d.query("> colgroup", node).map(function(cg){
				var sv = d.attr(cg, "span");
				var v = {
					noscroll: (d.attr(cg, "noscroll") == "true") ? true : false,
					__span: (!!sv ? parseInt(sv) : 1),
					cells: []
				};
				if(d.hasAttr(cg, "width")){
					v.width = widthFromAttr(cg);
				}
				return v; // for vendetta
			});
			if(!props.structure.length){
				props.structure.push({
					__span: Infinity,
					cells: [] // catch-all view
				});
			}
			// check to see if we're gonna have more than one view

			// for each tr in our th, create a row of cells
			d.query("thead > tr", node).forEach(function(tr, tr_idx){
				var cellCount = 0;
				var viewIdx = 0;
				var lastViewIdx;
				var cView = null;
				d.query("> th", tr).map(function(th){
					// what view will this cell go into?

					// NOTE:
					//		to prevent extraneous iteration, we start counters over
					//		for each row, incrementing over the surface area of the
					//		structure that colgroup processing generates and
					//		creating cell objects for each <th> to place into those
					//		cell groups.  There's a lot of state-keepking logic
					//		here, but it is what it has to be.
					if(!cView){ // current view book keeping
						lastViewIdx = 0;
						cView = props.structure[0];
					}else if(cellCount >= (lastViewIdx+cView.__span)){
						viewIdx++;
						// move to allocating things into the next view
						lastViewIdx += cView.__span;
						lastView = cView;
						cView = props.structure[viewIdx];
					}

					// actually define the cell from what markup hands us
					var cell = {
						name: d.trim(d.attr(th, "name")||th.innerHTML),
						colSpan: parseInt(d.attr(th, "colspan")||1, 10),
						type: d.trim(d.attr(th, "cellType")||"")
					};
					cellCount += cell.colSpan;
					var rowSpan = d.attr(th, "rowspan");
					if(rowSpan){
						cell.rowSpan = rowSpan;
					}
					if(d.hasAttr(th, "width")){
						cell.width = widthFromAttr(th);
					}
					if(d.hasAttr(th, "relWidth")){
						cell.relWidth = window.parseInt(dojo.attr(th, "relWidth"), 10);
					}
					if(d.hasAttr(th, "hidden")){
						cell.hidden = d.attr(th, "hidden") == "true";
					}

					if(cellFunc){
						cellFunc(th, cell);
					}

					cell.type = cell.type ? dojo.getObject(cell.type) : dojox.grid.cells.Cell;

					if(cell.type && cell.type.markupFactory){
						cell.type.markupFactory(th, cell);
					}

					if(!cView.cells[tr_idx]){
						cView.cells[tr_idx] = [];
					}
					cView.cells[tr_idx].push(cell);
				});
			});
		}

		return new ctor(props, node);
	}
})();
