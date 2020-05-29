dojo.provide("dojox.widget.RollingList");
dojo.experimental("dojox.widget.RollingList");

dojo.require("dijit._Templated");
dojo.require("dijit.layout.ContentPane");
dojo.require("dijit.Menu");
dojo.require("dojox.html.metrics");

dojo.require("dojo.i18n"); 
dojo.requireLocalization("dojox.widget", "RollingList"); 

dojo.declare("dojox.widget._RollingListPane",
	[dijit.layout.ContentPane, dijit._Templated, dijit._Contained], {
	// summary: a core pane that can be attached to a RollingList.  All panes
	//  should extend this one

	// templateString: string
	//	our template
	templateString: '<div class="dojoxRollingListPane"><table><tbody><tr><td dojoAttachPoint="containerNode"></td></tr></tbody></div>',

	// parentWidget: dojox.widget.RollingList
	//  Our rolling list widget
	parentWidget: null,
	
	// parentPane: dojox.widget._RollingListPane
	//  The pane that immediately precedes ours
	parentPane: null,
			
	// store: store
	//  the store we must use
	store: null,

	// items: item[]
	//  an array of (possibly not-yet-loaded) items to display in this.
	//  If this array is null, then the query and query options are used to
	//  get the top-level items to use.  This array is also used to watch and
	//  see if the pane needs to be reloaded (store notifications are handled)
	//  by the pane
	items: null,
	
	// query: object
	//  a query to pass to the datastore.  This is only used if items are null
	query: null,
	
	// queryOptions: object
	//  query options to be passed to the datastore
	queryOptions: null,
	
	// focusByNode: boolean
	//  set to false if the subclass will handle its own node focusing
	_focusByNode: true,
	
	_setContentAndScroll: function(/*String|DomNode|Nodelist*/cont){
		// summary: sets the value of the content and scrolls it into view
		this._setContent(cont);
		this.parentWidget.scrollIntoView(this);
	},

	startup: function(){
		if(this._started){ return; }
		if(this.store && this.store.getFeatures()["dojo.data.api.Notification"]){
			window.setTimeout(dojo.hitch(this, function(){
				// Set connections after a slight timeout to avoid getting in the
				// condition where we are setting them while events are still 
				// being fired
				this.connect(this.store, "onSet", "_onSetItem");
				this.connect(this.store, "onNew", "_onNewItem");
				this.connect(this.store, "onDelete", "_onDeleteItem");
			}), 1);
		}
		this.connect(this.focusNode||this.domNode, "onkeypress", "_focusKey");
		this.parentWidget._updateClass(this.domNode, "Pane");
		this.inherited(arguments);
	},

	_focusKey: function(/*Event*/e){
		// summary: called when a keypress happens on the widget
		if(e.charOrCode == dojo.keys.BACKSPACE){
			dojo.stopEvent(e);
			return;
		}else if(e.charOrCode == dojo.keys.LEFT_ARROW && this.parentPane){
			this.parentPane.focus();
			this.parentWidget.scrollIntoView(this.parentPane);
		}else if(e.charOrCode == dojo.keys.ENTER){
			this.parentWidget.onExecute();
		}
	},
	
	focus: function(/*boolean*/force){
		// summary: sets the focus to this current widget
		if(this.parentWidget._focusedPane != this){
			this.parentWidget._focusedPane = this;
			this.parentWidget.scrollIntoView(this);
			if(this._focusByNode && (!this.parentWidget._savedFocus || force)){
				try{(this.focusNode||this.domNode).focus();}catch(e){}
			}
		}
	},
	
	_loadCheck: function(/* Boolean? */ forceLoad){
		// summary: checks that the store is loaded
		if(!this._started){
			var c = this.connect(this, "startup", function(){
				this.disconnect(c);
				this._loadCheck(forceLoad);
			});
		}
		var displayState = this.domNode && this._isShown();
		if((this.store || this.items) && (forceLoad || (this.refreshOnShow && displayState) || (!this.isLoaded && displayState))){
			this._doQuery();
		}
	},
	
	_doQuery: function(){
		// summary: either runs the query or loads potentially not-yet-loaded items.
		this.isLoaded = false;
		if(this.items){
			var _waitCount = 0, store = this.store, items = this.items;
			dojo.forEach(items, function(item){ 
				if(!store.isItemLoaded(item)){ _waitCount++; }
			});
			if(_waitCount === 0){
				this.onItems();
			}else{
				var onItem = dojo.hitch(this, function(item){
					_waitCount--;
					if((_waitCount) === 0){
						this.onItems();
					}
				});
				this._setContentAndScroll(this.onLoadStart());
				dojo.forEach(items, function(item){
					if(!store.isItemLoaded(item)){
						store.loadItem({item: item, onItem: onItem});
					}
				});
			}
		}else{
			this._setContentAndScroll(this.onFetchStart());
			this.store.fetch({query: this.query, 
				onComplete: function(items){
					this.items = items;
					this.onItems();
				}, 
				onError: function(e){
					this._onError("Fetch", e);
				},
				scope: this});
		}
	},

	_hasItem: function(/* item */ item){
		// summary: returns whether or not the given item is handled by this 
		//  pane
		var items = this.items || [];
		for(var i = 0, myItem; (myItem = items[i]); i++){
			if(this.parentWidget._itemsMatch(myItem, item)){
				return true;
			}
		}
		return false;
	},
	
	_onSetItem: function(/* item */ item, 
					/* attribute-name-string */ attribute, 
					/* object | array */ oldValue,
					/* object | array */ newValue){	
		// Summary: called when an item in the store has changed
		if(this._hasItem(item)){
			this._loadCheck(true);
		}
	},
	
	_onNewItem: function(/* item */ newItem, /*object?*/ parentInfo){
		// Summary: called when an item is added to the store
		var sel;
		if((!parentInfo && !this.parentPane) ||
			(parentInfo && this.parentPane && this.parentPane._hasItem(parentInfo.item) &&
			(sel = this.parentPane._getSelected()) && this.parentWidget._itemsMatch(sel.item, parentInfo.item))){
			this.items.push(newItem);
			this._loadCheck(true);
		}else if(parentInfo && this.parentPane && this._hasItem(parentInfo.item)){
			this._loadCheck(true);
		}
	},
	
	_onDeleteItem: function(/* item */ deletedItem){
		// Summary: called when an item is removed from the store
		if(this._hasItem(deletedItem)){
			this.items = dojo.filter(this.items, function(i){
				return (i != deletedItem);
			});
			this._loadCheck(true);
		}
	},
	
	onFetchStart: function(){
		// summary:
		//		called before a fetch starts
		return this.loadingMessage;
	},
	
	onFetchError: function(/*Error*/ error){
		// summary:
		//	called when a fetch error occurs.
		return this.errorMessage;
	},

	onLoadStart: function(){
		// summary:
		//		called before a load starts
		return this.loadingMessage;
	},
	
	onLoadError: function(/*Error*/ error){
		// summary:
		//	called when a load error occurs.
		return this.errorMessage;
	},
	
	onItems: function(){
		// summary:
		//	called after a fetch or load - at this point, this.items should be
		//  set and loaded.  Override this function to "do your stuff"
		this._onLoadHandler();		
	}
			
});

dojo.declare("dojox.widget._RollingListGroupPane",
	[dojox.widget._RollingListPane], {
	// summary: a pane that will handle groups (treats them as menu items)
	
	// templateString: string
	//	our template
	templateString: '<div><div dojoAttachPoint="containerNode"></div>' +
					'<div dojoAttachPoint="menuContainer">' +
						'<div dojoAttachPoint="menuNode"></div>' +
					'</div></div>',

	// _menu: dijit.Menu
	//  The menu that we will call addChild() on for adding items
	_menu: null,
	
	_loadCheck: function(/* Boolean? */ forceLoad){
		// summary: checks that the store is loaded
		var displayState = this._isShown();
		if((this.store || this.items) && (forceLoad || (this.refreshOnShow && displayState) || (!this.isLoaded && displayState))){
			this._doQuery();
		}
	},
	
	_setContent: function(/*String|DomNode|Nodelist*/cont){
		if(!this._menu){
			// Only set the content if we don't already have a menu
			this.inherited(arguments);
		}
	},

	onItems: function(){
		// summary:
		//	called after a fetch or load - at this point, this.items should be
		//  set and loaded.
		var selectItem, hadChildren = false;
		if(this._menu){
			selectItem = this._getSelected();
			this._menu.destroyRecursive();
		}
		this._menu = this._getMenu();
		var child, selectMenuItem;
		if(this.items.length){
			dojo.forEach(this.items, function(item){
				child = this.parentWidget._getMenuItemForItem(item, this);
				if(child){
					if(selectItem && this.parentWidget._itemsMatch(child.item, selectItem.item)){
						selectMenuItem = child;
					}
					this._menu.addChild(child);
				}
			}, this);
		}else{
			child = this.parentWidget._getMenuItemForItem(null, this);
			if(child){
				this._menu.addChild(child);
			}
		}
		if(selectMenuItem){
			this._setSelected(selectMenuItem);
			if((selectItem && !selectItem.children && selectMenuItem.children) ||
				(selectItem && selectItem.children && !selectMenuItem.children)){
				var itemPane = this.parentWidget._getPaneForItem(selectMenuItem.item, this, selectMenuItem.children);
				if(itemPane){
					this.parentWidget.addChild(itemPane, this.getIndexInParent() + 1);
				}else{
					this.parentWidget._removeAfter(this);
					this.parentWidget._onItemClick(null, this, selectMenuItem.item, selectMenuItem.children);
				}
			}
		}else if(selectItem){
			this.parentWidget._removeAfter(this);
		}
		this.containerNode.innerHTML = "";
		this.containerNode.appendChild(this._menu.domNode);
		this.parentWidget.scrollIntoView(this);
		this.inherited(arguments);
	},
	
	startup: function(){
		this.inherited(arguments);
		this.parentWidget._updateClass(this.domNode, "GroupPane");
	},
	
	focus: function(/*boolean*/force){
		// summary: sets the focus to this current widget
		if(this._menu){
			if(this._pendingFocus){
				this.disconnect(this._pendingFocus);
			}
			delete this._pendingFocus;
			
			// We focus the right widget - either the focusedChild, the
			//   selected node, the first menu item, or the menu itself
			var focusWidget = this._menu.focusedChild;
			if(!focusWidget){
				var focusNode = dojo.query(".dojoxRollingListItemSelected", this.domNode)[0];
				if(focusNode){
					focusWidget = dijit.byNode(focusNode);
				}
			}
			if(!focusWidget){
				focusWidget = this._menu.getChildren()[0] || this._menu;
			}
			this._focusByNode = false;
			if(focusWidget.focusNode){
				if(!this.parentWidget._savedFocus || force){
					try{focusWidget.focusNode.focus();}catch(e){}
				}
				window.setTimeout(function(){
					try{
						dijit.scrollIntoView(focusWidget.focusNode);
					}catch(e){}
				}, 1);
			}else if(focusWidget.focus){
				if(!this.parentWidget._savedFocus || force){
					focusWidget.focus();
				}
			}else{
				this._focusByNode = true;
			}
			this.inherited(arguments);
		}else if(!this._pendingFocus){
			this._pendingFocus = this.connect(this, "onItems", "focus");
		}
	},
	
	_getMenu: function(){
		// summary: returns a widget to be used for the container widget.
		var self = this;
		var menu = new dijit.Menu({
			parentMenu: this.parentPane ? this.parentPane._menu : null,
			onCancel: function(/*Boolean*/ closeAll){ 
				if(self.parentPane){
					self.parentPane.focus(true);
				}
			},
			_moveToPopup: function(/*Event*/ evt){
				if(this.focusedChild && !this.focusedChild.disabled){
					this.focusedChild._onClick(evt);
				}
			}
		}, this.menuNode);
		this.connect(menu, "onItemClick", function(/*dijit.MenuItem*/ item, /*Event*/ evt){
			if(item.disabled){ return; }
			evt.alreadySelected = dojo.hasClass(item.domNode, "dojoxRollingListItemSelected");
			if(evt.alreadySelected && 
				((evt.type == "keypress" && evt.charOrCode != dojo.keys.ENTER) ||
				(evt.type == "internal"))){
				var p = this.parentWidget.getChildren()[this.getIndexInParent() + 1];
				if(p){
					p.focus(true);
					this.parentWidget.scrollIntoView(p);
				}
			}else{
				this._setSelected(item, menu);
				this.parentWidget._onItemClick(evt, this, item.item, item.children);
				if(evt.type == "keypress" && evt.charOrCode == dojo.keys.ENTER){
					this.parentWidget.onExecute();
				}
			}
		});
		if(!menu._started){
			menu.startup();
		}
		return menu;
	},
	
	_getSelected: function(/*dijit.Menu?*/ menu){
		// summary:
		//	returns the selected menu item - or null if none are selected
		if(!menu){ menu = this._menu; }
		if(menu){
			var children = this._menu.getChildren();
			for(var i = 0, item; (item = children[i]); i++){
				if(dojo.hasClass(item.domNode, "dojoxRollingListItemSelected")){
					return item;
				}
			}
		}
		return null;
	},
	
	_setSelected: function(/*dijit.MenuItem?*/ item, /*dijit.Menu?*/ menu){
		// summary:
		//	selectes the given item in the given menu (defaults to pane's menu)
		if(!menu){ menu = this._menu;}
		if(menu){
			dojo.forEach(menu.getChildren(), function(i){
				this.parentWidget._updateClass(i.domNode, "Item", {"Selected": (item && (i == item && !i.disabled))});
			}, this);
		}
	}
});

dojo.declare("dojox.widget.RollingList",
	[dijit._Widget, dijit._Templated, dijit._Container], {
	// summary: a rolling list that can be tied to a data store with children
		
	// templateString: string
	//  our template string to use
	templateString: '<div class="dojoxRollingList ${className}" dojoAttachPoint="containerNode" dojoAttachEvent="onkeypress:_onKey"></div>',
	
	// className: string
	//  an additional class (or space-separated classes) to add for our widget
	className: "",
	
	// store: store
	//  the store we must use
	store: null,
	
	// query: object
	//  a query to pass to the datastore.  This is only used if items are null
	query: null,
	
	// queryOptions: object
	//  query options to be passed to the datastore
	queryOptions: null,
	
	// childrenAttrs: String[]
	//		one ore more attributes that holds children of a node
	childrenAttrs: ["children"],

	// parentAttr: string
	//	the attribute to read for finding our parent item (if any)
	parentAttr: "",
	
	// value: item
	//		The value that has been selected
	value: null,

	_itemsMatch: function(/*item*/ item1, /*item*/ item2){
		// Summary: returns whether or not the two items match - checks ID if
		//  they aren't the exact same object
		if(!item1 && !item2){ 
			return true;
		}else if(!item1 || !item2){
			return false;
		}
		return (item1 == item2 || 
			(this._isIdentity && this.store.getIdentity(item1) == this.store.getIdentity(item2)));
	},
	
	_removeAfter: function(/*Widget or int*/ idx){
		// summary: removes all widgets after the given widget (or index)
		if(typeof idx != "number"){
			idx = this.getIndexOfChild(idx);
		}
		if(idx >= 0){
			dojo.forEach(this.getChildren(), function(c, i){
				if(i > idx){
					this.removeChild(c);
					c.destroyRecursive();
				}
			}, this);
		}
		var children = this.getChildren(), child = children[children.length - 1];
		var selItem = null;
		while(child && !selItem){
			var val = child._getSelected ? child._getSelected() : null;
			if(val){
				selItem = val.item;
			}
			child = child.parentPane;
		}
		if(!this._setInProgress){
			this._setValue(selItem);
		}
	},
	
	addChild: function(/*Widget*/ widget, /*int?*/ insertIndex){
		// summary: adds a child to this rolling list - if passed an insertIndex,
		//  then all children from that index on will be removed and destroyed
		//  before adding the child.
		if(insertIndex > 0){
			this._removeAfter(insertIndex - 1);
		}
		this.inherited(arguments);
		if(!widget._started){
			widget.startup();
		}
		this.layout();
		if(!this._savedFocus){
			widget.focus();
		}
	},
	
	_updateClass: function(/* Node */ node, /* String */ type, /* Object? */ options){
		// summary: 
		//		sets the state of the given node with the given type and options
		// options: 
		//		an object with key-value-pairs.  The values are boolean, if true,
		//		the key is added as a class, if false, it is removed.
		if(!this._declaredClasses){
			this._declaredClasses = ("dojoxRollingList " + this.className).split(" ");
		}
		dojo.forEach(this._declaredClasses, function(c){
			if(c){
				dojo.addClass(node, c + type);
				for(var k in options||{}){
					dojo.toggleClass(node, c + type + k, options[k]);
				}
				dojo.toggleClass(node, c + type + "FocusSelected", 
					(dojo.hasClass(node, c + type + "Focus") && dojo.hasClass(node, c + type + "Selected")));
				dojo.toggleClass(node, c + type + "HoverSelected", 
					(dojo.hasClass(node, c + type + "Hover") && dojo.hasClass(node, c + type + "Selected")));
			}
		});
	},
	
	scrollIntoView: function(/* Widget */ childWidget){
		// summary: scrolls the given widget into view
		if(this._scrollingTimeout){ 
			window.clearTimeout(this._scrollingTimeout);
		}
		delete this._scrollingTimeout;
		this._scrollingTimeout = window.setTimeout(dojo.hitch(this, function(){
			if(childWidget.domNode){
				dijit.scrollIntoView(childWidget.domNode);
			}
			delete this._scrollingTimeout;
			return;
		}), 1);
	},
	
	resize: function(args){
		dijit.layout._LayoutWidget.prototype.resize.call(this, args);
	},
	
	layout: function(){
		var children = this.getChildren();
		if(this._contentBox){
			var height = this._contentBox.h - dojox.html.metrics.getScrollbar().h;
			dojo.forEach(children, function(c){
				dojo.marginBox(c.domNode, {h: height});
			});
		}
		if(this._focusedPane){
			var foc = this._focusedPane;
			delete this._focusedPane;
			if(!this._savedFocus){
				foc.focus();
			}
		}else if(children && children.length){
			if(!this._savedFocus){
				children[0].focus();
			}
		}
	},
	
	_onChange: function(/*item*/ value){
		this.onChange(value);
	},

	_setValue: function(/* item */ value){
		// summary: internally sets the value and fires onchange
		delete this._setInProgress;
		if(!this._itemsMatch(this.value, value)){
			this.value = value;
			this._onChange(value);
		}
	},
	
	_setValueAttr: function(/* item */ value){
		// summary: sets the value of this widget to the given store item
		if(this._itemsMatch(this.value, value) && !value){ return; }
		if(this._setInProgress && this._setInProgress === value){ return; }
		this._setInProgress = value;
		if(!value || !this.store.isItem(value)){
			var pane = this.getChildren()[0];
			pane._setSelected(null);
			this._onItemClick(null, pane, null, null);
			return;
		}
		
		var fetchParentItems = dojo.hitch(this, function(/*item*/ item, /*function*/callback){
			// Summary: Fetchs the parent items for the given item
			var store = this.store, id;
			if(this.parentAttr && store.getFeatures()["dojo.data.api.Identity"] &&
				((id = this.store.getValue(item, this.parentAttr)) || id === "")){
				// Fetch by parent attribute
				var cb = function(i){
					if(store.getIdentity(i) == store.getIdentity(item)){
						callback(null);
					}else{
						callback([i]);
					}
				};
				if(id === ""){
					callback(null);
				}else if(typeof id == "string"){
					store.fetchItemByIdentity({identity: id, onItem: cb});
				}else if(store.isItem(id)){
					cb(id);
				}
			}else{
				// Fetch by finding children
				var numCheck = this.childrenAttrs.length;
				var parents = [];
				dojo.forEach(this.childrenAttrs, function(attr){
					var q = {};
					q[attr] = item;
					store.fetch({query: q, scope: this, 
						onComplete: function(items){
							if(this._setInProgress !== value){
								return;
							}
							parents = parents.concat(items);
							numCheck--;
							if(numCheck === 0){
								callback(parents);
							}
						}
					});
				}, this);
			}
		});
		
		var setFromChain = dojo.hitch(this, function(/*item[]*/itemChain, /*integer*/idx){
			// Summary: Sets the value of the widget at the given index in the chain - onchanges are not 
			// fired here
			var set = itemChain[idx];
			var child = this.getChildren()[idx];
			var conn;
			if(set && child){
				var fx = dojo.hitch(this, function(){
					if(conn){
						this.disconnect(conn);
					}
					delete conn;
					if(this._setInProgress !== value){
						return;
					}
					var selOpt = dojo.filter(child._menu.getChildren(), function(i){
						return this._itemsMatch(i.item, set);
					}, this)[0];
					if(selOpt){
						idx++;
						child._menu.onItemClick(selOpt, {type: "internal",
													stopPropagation: function(){},
													preventDefault: function(){}});
						if(itemChain[idx]){
							setFromChain(itemChain, idx);
						}else{
							this._setValue(set);
							this.onItemClick(set, child, this.getChildItems(set));
						}
					}
				});
				if(!child.isLoaded){
					conn = this.connect(child, "onLoad", fx);
				}else{
					fx();
				}
			}else if(idx === 0){
				this.attr("value", null);
			}
		});
		
		var parentChain = [];
		var onParents = dojo.hitch(this, function(/*item[]*/ parents){
			// Summary: recursively grabs the parents - only the first one is followed
			if(parents && parents.length){
				parentChain.push(parents[0]);
				fetchParentItems(parents[0], onParents);
			}else{
				if(!parents){
					parentChain.pop();
				}
				parentChain.reverse();
				setFromChain(parentChain, 0);
			}
		});
		
		// Only set the value in display if we are shown - if we are in a dropdown, 
		// and are hidden, don't actually do the scrolling in the display (it can
		// mess up layouts)
		var ns = this.domNode.style;
		if(ns.display == "none" || ns.visibility == "hidden"){
			this._setValue(value);
		}else if(!this._itemsMatch(value, this._visibleItem)){
			onParents([value]);
		}
	},
	
	_onItemClick: function(/* Event */ evt, /* dijit._Contained */ pane, /* item */ item, /* item[]? */ children){
		// summary: internally called when a widget should pop up its child
		
		if(evt){
			var itemPane = this._getPaneForItem(item, pane, children);
			var alreadySelected = (evt.type == "click" && evt.alreadySelected);

			if(alreadySelected && itemPane){
				this._removeAfter(pane.getIndexInParent() + 1);
				var next = pane.getNextSibling();
				if(next && next._setSelected){
					next._setSelected(null);
				}
				this.scrollIntoView(next);
			}else if(itemPane){
				this.addChild(itemPane, pane.getIndexInParent() + 1);
				if(this._savedFocus){
					itemPane.focus(true);
				}
			}else{
				this._removeAfter(pane);
				this.scrollIntoView(pane);
			}
		}else if(pane){
			this._removeAfter(pane);
			this.scrollIntoView(pane);
		}
		if(!evt || evt.type != "internal"){
			this._setValue(item);
			this.onItemClick(item, pane, children);
		}
		this._visibleItem = item;
	},
	
	_getPaneForItem: function(/* item? */ item, /* dijit._Contained? */ parentPane, /* item[]? */ children){		// summary: gets the pane for the given item, and mixes in our needed parts
		// Returns the pane for the given item (null if the root pane) - after mixing in
		// its stuff.
		var ret = this.getPaneForItem(item, parentPane, children);
		ret.store = this.store;
		ret.parentWidget = this;
		ret.parentPane = parentPane||null;
		if(!item){
			ret.query = this.query;
			ret.queryOptions = this.queryOptions;
		}else if(children){
			ret.items = children;
		}else{
			ret.items = [item];
		}
		return ret;
	},
	
	_getMenuItemForItem: function(/*item*/ item, /* dijit._Contained */ parentPane){
		// summary: returns a widget for the given store item.  The returned
		//  item will be added to this widget's container widget.  null will
		//  be passed in for an "empty" item.
		var store = this.store;
		if(!item || !store && !store.isItem(item)){
			var i = new dijit.MenuItem({
				label: dojo.i18n.getLocalization("dojox.widget", "RollingList", this.lang).empty,
				disabled: true,
				iconClass: "dojoxEmpty",
				focus: function(){
					// Do nothing on focus of this guy...
				}
			});	
			this._updateClass(i.domNode, "Item");
			return i;
		}else{
			var childItems = this.getChildItems(item);
			var widgetItem;
			if(childItems){
				widgetItem = this.getMenuItemForItem(item, parentPane, childItems);
				widgetItem.children = childItems;
				this._updateClass(widgetItem.domNode, "Item", {"Expanding": true});
				if(!widgetItem._started){
					var c = widgetItem.connect(widgetItem, "startup", function(){
						this.disconnect(c);
						dojo.style(this.arrowWrapper, "display", "");
					});
				}else{
					dojo.style(widgetItem.arrowWrapper, "display", "");
				}
			}else{
				widgetItem = this.getMenuItemForItem(item, parentPane, null);
				this._updateClass(widgetItem.domNode, "Item", {"Single": true});
			}
			widgetItem.store = this.store;
			widgetItem.item = item;
			if(!widgetItem.label){
				widgetItem.attr("label", this.store.getLabel(item));
			}
			if(widgetItem.focusNode){
				var self = this;
				widgetItem.focus = function(){
					// Don't set our class
					if(!this.disabled){try{this.focusNode.focus();}catch(e){}}
				};
				widgetItem.connect(widgetItem.focusNode, "onmouseenter", function(){
					self._updateClass(this.domNode, "Item", {"Hover": true});
				});
				widgetItem.connect(widgetItem.focusNode, "onmouseleave", function(){
					self._updateClass(this.domNode, "Item", {"Hover": false});
				});
				widgetItem.connect(widgetItem.focusNode, "blur", function(){
					self._updateClass(this.domNode, "Item", {"Focus": false});
				});
				widgetItem.connect(widgetItem.focusNode, "focus", function(){
					self._updateClass(this.domNode, "Item", {"Focus": true});
					self._focusedPane = parentPane;
				});
				widgetItem.connect(widgetItem.focusNode, "ondblclick", function(){
					self.onExecute();
				});
			}
			return widgetItem;
		}
	},
	
	_setStore: function(/* dojo.data.api.Read */ store){
		// summary: sets the store for this widget */
		if(store === this.store && this._started){ return; }
		this.store = store;
		this._isIdentity = store.getFeatures()["dojo.data.api.Identity"];
		var rootPane = this._getPaneForItem();
		this.addChild(rootPane, 0);
	},
	
	_onKey: function(/*Event*/ e){
		// summary: called when a keypress event happens on this widget
		if(e.charOrCode == dojo.keys.BACKSPACE){
			dojo.stopEvent(e);
			return;
		}else if(e.charOrCode == dojo.keys.ESCAPE && this._savedFocus){
			try{dijit.focus(this._savedFocus);}catch(e){}
			dojo.stopEvent(e);
			return;
		}else if(e.charOrCode == dojo.keys.LEFT_ARROW || 
			e.charOrCode == dojo.keys.RIGHT_ARROW){
			dojo.stopEvent(e);
			return;
		}
	},
	
	focus: function(){
		// summary: sets the focus state of this widget
		var wasSaved = this._savedFocus;
		this._savedFocus = dijit.getFocus(this);
		if(!this._savedFocus.node){
			delete this._savedFocus;
		}
		if(!this._focusedPane){
			var child = this.getChildren()[0];
			if(child && !wasSaved){
				child.focus(true);
			}
		}else{
			this._savedFocus = dijit.getFocus(this);
			var foc = this._focusedPane;
			delete this._focusedPane;
			if(!wasSaved){
				foc.focus(true);
			}
		}
	},
	
	handleKey:function(/*Event*/e){
		// summary: handle the key for the given event - called by dropdown
		//	widgets
		if(e.charOrCode == dojo.keys.DOWN_ARROW){
			delete this._savedFocus;
			this.focus();
			return false;
		}else if(e.charOrCode == dojo.keys.ESCAPE){
			this.onCancel();
			return false;
		}
		return true;
	},

	startup: function(){
		if(this._started){ return; }
		if(!this.getParent || !this.getParent()){
			this.resize();
			this.connect(dojo.global, "onresize", "resize");
		}
		this._setStore(this.store);
		this.inherited(arguments);
	},
	
	getChildItems: function(/*item*/ item){
		// summary: Returns the child items for the given store item
		var childItems, store = this.store;
		dojo.forEach(this.childrenAttrs, function(attr){
			var vals = store.getValues(item, attr);
			if(vals && vals.length){
				childItems = (childItems||[]).concat(vals);
			}
		});
		return childItems;
	},
	
	getMenuItemForItem: function(/*item*/ item, /* dijit._Contained */ parentPane, /* item[]? */ children){
		// summary: user overridable function to return a widget for the given item
		//  and its children.
		return new dijit.MenuItem({});
	},

	getPaneForItem: function(/* item? */ item, /* dijit._Contained? */ parentPane, /* item[]? */ children){
		// summary: user-overridable function to return a pane that corresponds
		//  to the given item in the store.  It can return null to not add a new pane
		//  (ie, you are planning on doing something else with it in onItemClick)
		//
		//  Item is undefined for the root pane, children is undefined for non-group panes 
		if(!item || children){
			return new dojox.widget._RollingListGroupPane({});
		}else{
			return null;
		}
	},

	onItemClick: function(/* item */ item, /* dijit._Contained */ pane, /* item[]? */ children){
		// summary: called when an item is clicked - it receives the store item
	},
	
	onExecute: function(){
		// summary: exists so that popups don't disappear too soon
	},
	
	onCancel: function(){
		// summary: exists so that we can close ourselves if we wish
	},
	
	onChange: function(/* item */ value){
		// summary: called when the value of this widget has changed
	}
	
});