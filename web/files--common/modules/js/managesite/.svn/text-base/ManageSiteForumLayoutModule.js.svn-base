/*
 * Wikidot - free wiki collaboration software
 * Copyright (c) 2008, Wikidot Inc.
 * 
 * Code licensed under the GNU Affero General Public 
 * License version 3 or later.
 *
 * For more information about licensing visit:
 * http://www.wikidot.org/license
 */

// ok, make some things global here:

var structureArray = new Object();
structureArray['0'] = "0 (flat/linear)";
structureArray['1'] = "1";
structureArray['2'] = "2";
structureArray['3'] = "3";
structureArray['4'] = "4";
structureArray['5'] = "5";
structureArray['6'] = "6";
structureArray['7'] = "7";
structureArray['8'] = "8";
structureArray['9'] = "9";
structureArray['10'] = "10";
structureArray[''] = "forum default";

// each entry should be an array with some properties set.
var groups = new Array();

var categories = new Array();

var deletedGroups = new Array();
var deletedCategories = new Array();

var defaultNestingLevel;

WIKIDOT.modules.ManageSiteForumLayoutModule = {};

WIKIDOT.modules.ManageSiteForumLayoutModule.vars = {};

WIKIDOT.modules.ManageSiteForumLayoutModule.listeners = {
	newGroup: function(e){
		// show new group form
		var el = $("new-group-window");
		var w = new OZONE.dialogs.Dialog();
		w.content = el.innerHTML.replace(/template\-id\-stub\-/g, 'a-');
		w.show();
		
	},
	
	editGroup: function(groupIndex){
		var el = $("new-group-window");
		var w = new OZONE.dialogs.Dialog();
		w.content =el.innerHTML.replace(/template\-id\-stub\-/g, 'a-');
		w.show();
		var group = groups[groupIndex];
		$("a-group-name").value = group['name'];
		$("a-gindex").value = groupIndex;
		$("a-group-description").value = group['description'];
		
	},
	
	saveGroup: function(e){
		var name = $("a-group-name").value;
		var description =  $("a-group-description").value;
		// validate please...
		var errors = new Array();
		if(name.length == 0){
			errors[errors.length] = "The name should not be empty";
		}
		
		if(errors.length >0){
			// form HAS errors. print them and exit the function
			$("a-form-error-list").innerHTML = errors.join('<br/>');
			$("a-form-error-container").style.display="block";
		} else {
			// the form is ok, create a new group here...
			var gIndex = $("a-gindex").value;
			var group;
			if(gIndex == "" || gIndex == null){
				group = new Object();
			} else{
				var group = groups[gIndex];
			}
			group['name'] = name;
			group['description'] = description;
			group['visible'] = true;
			if(gIndex == "" || gIndex == null){
				groups[groups.length] = group;
				categories[groups.length-1] = new Array();
			}else{
				groups[gIndex] = group;
			}
			WIKIDOT.modules.ManageSiteForumLayoutModule.utils.refreshDisplay();
			OZONE.dialog.cleanAll();
		}

	},
	
	hideGroup: function(groupIndex){
		groups[groupIndex]['visible']=false;
		WIKIDOT.modules.ManageSiteForumLayoutModule.utils.refreshDisplay();
	},
	
	showGroup: function(groupIndex){
		groups[groupIndex]['visible']=true;
		WIKIDOT.modules.ManageSiteForumLayoutModule.utils.refreshDisplay();
	},
	
	deleteGroup: function(groupIndex){
		// check if not empty
		if(categories[groupIndex].length > 0){
			var w = new OZONE.dialogs.ErrorDialog();
			w.content = "A non-empty forum group can not be deleted.";
			w.show();
		} else {
			deletedGroups.push(groups[groupIndex]);
			groups.splice(groupIndex,1);
			categories.splice(groupIndex,1);
			WIKIDOT.modules.ManageSiteForumLayoutModule.utils.refreshDisplay();
		}
	},
	
	deleteCategory: function(groupIndex, categoryIndex){
		var category =  categories[groupIndex][categoryIndex];
		if(category['number_threads'] && category['number_threads']>0){
			var w = new OZONE.dialogs.ErrorDialog();
			w.content = "A non-empty category can not be deleted. Consider rather moving the category " +
					"to a hidden group.";
			w.show();
		}else{
			if(category['category_id']){
				deletedCategories.push(category['category_id']);
			}
			categories[groupIndex].splice(categoryIndex,1);
			WIKIDOT.modules.ManageSiteForumLayoutModule.utils.refreshDisplay();
		}
	},
	
	addCategory: function(groupIndex){
		// show new group form
		var el = document.getElementById("new-category-window");
		var w = new OZONE.dialogs.Dialog();
		w.content= el.innerHTML.replace(/template\-id\-stub\-/g, 'a-').replace(/%%ACTION_TYPE%%/, 'Create a new');;
		w.show();
		$("a-group-index").value=groupIndex;
	},
	
	editCategory: function(groupIndex, categoryIndex){
		var cat = categories[groupIndex][categoryIndex];
		var el = $("new-category-window");
		var w = new OZONE.dialogs.Dialog();
		w.content=el.innerHTML.replace(/template\-id\-stub\-/g, 'a-').replace(/%%ACTION_TYPE%%/, 'Edit');
		w.show();
		$("a-gcategory-name").value = cat['name'];
		$("a-gcategory-description").value = cat['description'];
		$("a-category-index").value = categoryIndex;
		$("a-group-index").value=groupIndex;
		var mnl = cat['max_nest_level'];
		if(mnl == null){mnl = '';}
			$("a-gcategory-structure").value = mnl;

	},
	
	saveCategory: function(e){
		var name = $("a-gcategory-name").value;
		var description =  $("a-gcategory-description").value;
		var groupIndex = $("a-group-index").value;
		var maxNestLevel = $("a-gcategory-structure").value;
		
		// validate please...
		var errors = new Array();
		if(name.length == 0){
			errors[errors.length] = "The name should not be empty";
		}
		
		if(errors.length >0){
			// form HAS errors. print them and exit the function
			$("a-form-gerror-list").innerHTML = errors.join('<br/>');
			$("a-form-gerror-container").style.display="block";
		} else {
			// the form is ok, create a new group here...
			var categoryIndex = document.getElementById("a-category-index").value;
			var category;
			if(categoryIndex == "" || categoryIndex == null){
				category = new Object();
			} else {
				category = categories[groupIndex][categoryIndex];
			}
		 
			category['name'] = name;
			category['description'] = description;
			if(maxNestLevel == '') maxNestLevel = null;
			category['max_nest_level'] = maxNestLevel;
			
			if(categoryIndex == "" || categoryIndex == null){
				categories[groupIndex].push(category);
			}
			WIKIDOT.modules.ManageSiteForumLayoutModule.utils.refreshDisplay();
			OZONE.dialog.cleanAll();
		}

	},
	
	moveGroupUp: function(groupIndex){
		if(groupIndex > 0){
			var tmp1 = groups[groupIndex-1];
			groups[groupIndex-1] = groups[groupIndex];
			groups[groupIndex] = tmp1;
			
			tmp1 = categories[groupIndex-1];
			categories[groupIndex-1] = categories[groupIndex];
			categories[groupIndex] = tmp1;
			WIKIDOT.modules.ManageSiteForumLayoutModule.utils.refreshDisplay();
		}
		
	},
	moveGroupDown: function(groupIndex){
		if(groupIndex < groups.length-1){
			var tmp1 = groups[groupIndex+1];
			groups[groupIndex+1] = groups[groupIndex];
			groups[groupIndex] = tmp1;
			
			tmp1 = categories[groupIndex+1];
			categories[groupIndex+1] = categories[groupIndex];
			categories[groupIndex] = tmp1;
			WIKIDOT.modules.ManageSiteForumLayoutModule.utils.refreshDisplay();
		}
		
	},
	moveCategoryUp: function(groupIndex, categoryIndex){
		// move within one group or promote to another group...
		// if within group
		if(categoryIndex>0){
			var cats = categories[groupIndex];
			var tmp1 = cats[categoryIndex];
			cats[categoryIndex] = cats[categoryIndex-1];
			cats[categoryIndex-1] = tmp1;
		
		}else{
			// inter-group
			if(groupIndex >0){
				var cat = categories[groupIndex].shift();
				categories[groupIndex-1].push(cat);
			}
		}
		WIKIDOT.modules.ManageSiteForumLayoutModule.utils.refreshDisplay();
	},
	moveCategoryDown: function(groupIndex, categoryIndex){
		// move within one group or promote to another group...
		// if within group
		if(categoryIndex<categories[groupIndex].length - 1){
			var cats = categories[groupIndex];
			var tmp1 = cats[categoryIndex];
			cats[categoryIndex] = cats[categoryIndex+1];
			cats[categoryIndex+1] = tmp1;
		
		}else{
			// inter-group
			if(groupIndex < groups.length - 1){
				var cat = categories[groupIndex].pop();
				categories[groupIndex+1].splice(0,0,cat);
			}
		}
		WIKIDOT.modules.ManageSiteForumLayoutModule.utils.refreshDisplay();
	},
	cancel: function(e){
		WIKIDOT.modules.ManagerSiteModule.utils.loadModule('sm-welcome');
//		OZONE.ajax.requestModule("managesite/ManageSiteModule", null, WIKIDOT.modules.ManageSiteForumLayoutModule.callbacks.cancel)
	},
	save: function(e){
		var p = new Object();
		p['action'] = 'ManageSiteForumAction';
		p['event'] = 'saveForumLayout';
		p['groups'] = JSON.stringify(groups);
		p['categories'] = JSON.stringify(categories);
		p['deleted_groups'] = JSON.stringify(deletedGroups);
		p['deleted_categories'] = JSON.stringify(deletedCategories);
		OZONE.ajax.requestModule(null, p, WIKIDOT.modules.ManageSiteForumLayoutModule.callbacks.save);
		var w = new OZONE.dialogs.WaitBox();
		w.content = "Saving forum structure...";
		w.show();
	}
	
}

WIKIDOT.modules.ManageSiteForumLayoutModule.callbacks = {
	cancel: function(r){
		if(!WIKIDOT.utils.handleError(r)) {return;}
		OZONE.utils.setInnerHTMLContent("site-manager", r.body);
	},
	save: function(r){
		if(!WIKIDOT.utils.handleError(r)) {return;}
		var w = new OZONE.dialogs.SuccessBox();
		w.content = "Structure saved.";
		w.show();
		OZONE.ajax.requestModule("managesite/ManageSiteGetForumLayoutModule",null,WIKIDOT.modules.ManageSiteForumLayoutModule.callbacks.getLayout);
	},
	
	getLayout: function(r){
		if(!WIKIDOT.utils.handleError(r)) {return;}
		groups = r.groups;
		categories = r.categories;
		defaultNestingLevel = r.defaultNesting;
		WIKIDOT.modules.ManageSiteForumLayoutModule.utils.refreshDisplay();
	}
}

WIKIDOT.modules.ManageSiteForumLayoutModule.utils = {
	refreshDisplay: function(){
		var div = document.getElementById("layout-show-area");
		var inner = "";
		for(var i=0;i<groups.length;i++){
			var group = groups[i];

			inner += '<div class="sm-fgroup"';
			if(!group['visible']){inner+='style="color: #777" ';}
			inner += '>' +
					'<div class="sm-fgroup-name">'+OZONE.utils.escapeHtml(group['name']);
			if(!group['visible']){inner+= ' (hidden)';}
			inner +='</div>' +
					'<div class="sm-fgroup-description">'+OZONE.utils.escapeHtml(group['description'])+'</div>' +
					'<div class="sm-fgroup-options">' +
						'<a href="javascript:;" onclick="WIKIDOT.modules.ManageSiteForumLayoutModule.listeners.editGroup('+i+')">edit</a> |';
			if(!group['visible']){
				inner += '<a href="javascript:;" onclick="WIKIDOT.modules.ManageSiteForumLayoutModule.listeners.showGroup('+i+')">show</a> | ';
			} else {
				inner += '<a href="javascript:;" onclick="WIKIDOT.modules.ManageSiteForumLayoutModule.listeners.hideGroup('+i+')">hide</a> | ';
			}
			inner += '<a href="javascript:;" onclick="WIKIDOT.modules.ManageSiteForumLayoutModule.listeners.deleteGroup('+i+')">delete</a> |';
			inner +='<a href="javascript:;" onclick="WIKIDOT.modules.ManageSiteForumLayoutModule.listeners.addCategory('+i+')">add category</a> | ' +
			
						'<a href="javascript:;" onclick="WIKIDOT.modules.ManageSiteForumLayoutModule.listeners.moveGroupUp('+i+')">move up</a> | ' +
						'<a href="javascript:;" onclick="WIKIDOT.modules.ManageSiteForumLayoutModule.listeners.moveGroupDown('+i+')">move down</a></div>';
					
			// now add all categories...
			var cats = categories[i];
			for(var j=0; j<cats.length; j++){
				var cat = cats[j];
				if(!cat['number_threads']){
					cat['number_threads'] = 0;
				}
				inner += '<div class="sm-fcat">' +
						'<div class="sm-fcat-name">'+OZONE.utils.escapeHtml(cat['name'])+'</div>' +
						'<div class="sm-fcat-description">'+OZONE.utils.escapeHtml(cat['description'])+'</div>' +
						'<div class="sm-fcat-info">' +
						'number of threads: '+cat['number_threads']+'<br/>' +
						'maximum nesting level: ';
				if(cat['max_nest_level'] == null){
					inner += 'default forum nesting  ('+structureArray[defaultNestingLevel]+')';
				} else {
					inner += structureArray[cat['max_nest_level']];
				}
				inner += "<br/>";
				inner +=	'</div>' +
						'<div class="sm-fcat-options">' +
						'<a href="javascript:;" onclick="WIKIDOT.modules.ManageSiteForumLayoutModule.listeners.editCategory('+i+','+j+')">edit</a> | ' +
						'<a href="javascript:;" onclick="WIKIDOT.modules.ManageSiteForumLayoutModule.listeners.deleteCategory('+i+','+j+')">delete</a> | ' +
						'<a href="javascript:;" onclick="WIKIDOT.modules.ManageSiteForumLayoutModule.listeners.moveCategoryUp('+i+','+j+')">move up</a> | ' +
						'<a href="javascript:;" onclick="WIKIDOT.modules.ManageSiteForumLayoutModule.listeners.moveCategoryDown('+i+','+j+')">move down</a></div>' +
						'</div>';
			}
			inner+='</div>'; //close sm-fgroup div
		}
		div.innerHTML = inner;
	}
}

WIKIDOT.modules.ManageSiteForumLayoutModule.init = function(){
	YAHOO.util.Event.addListener("new-group-b", "click", WIKIDOT.modules.ManageSiteForumLayoutModule.listeners.newGroup);
	
	// get layout:
	
	OZONE.ajax.requestModule("managesite/ManageSiteGetForumLayoutModule",null,WIKIDOT.modules.ManageSiteForumLayoutModule.callbacks.getLayout);

}

WIKIDOT.modules.ManageSiteForumLayoutModule.init();
