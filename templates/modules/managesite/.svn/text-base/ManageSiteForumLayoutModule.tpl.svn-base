<h1>Forum structure</h1>

<div id="layout-show-area"></div>

<a href="javascript:;" id="new-group-b">create a new group</a>

<form>
	<div class="buttons">
		<input type="button" value="cancel" onclick="WIKIDOT.modules.ManageSiteForumLayoutModule.listeners.cancel(event)"/>
		<input type="button" value="save" onclick="WIKIDOT.modules.ManageSiteForumLayoutModule.listeners.save(event)"/>
	</div>
</form>



<div style="display: none" id="new-group-window">
	<div class="owindow">
		<div class="title">
			New forum group
		</div>
		<div class="content">	
			<h1>Create a new forum group</h1>
			<p>
				A category group is a collection of forum categories and the
				largest unit in the forum.
			</p>
			<div class="error-block" style="display: none" id="template-id-stub-form-error-container">
				Please correct the following errors:
				<div id="template-id-stub-form-error-list">
				</div>
			</div>
			<table class="form">
				<tr>
					<td>
						Group name:
					</td>
					<td>
						<input class="text" type="text" name="group-name" id="template-id-stub-group-name" size="40" maxlength="50"/>
					</td>
				</tr>
				<tr>
					<td>
						Short description:
					</td>
					<td>
						<textarea name="group-description" id="template-id-stub-group-description" cols="50" rows="3"></textarea>
					</td>
				</tr>
			</table>			
			<input type="hidden" id="template-id-stub-gindex"/>
		</div>
		<div class="button-bar">
			<a href="javascript:;" onclick="OZONE.dialog.cleanAll()">cancel</a>
			<a href="javascript:;" onclick="WIKIDOT.modules.ManageSiteForumLayoutModule.listeners.saveGroup(event)">save</a>
		</div>
	</div>
	
	

</div>

<div style="display: none" id="new-category-window">
	<div class="owindow">
		<div class="title">
			Forum category
		</div>
		<div class="content">	
			<h1>%%ACTION_TYPE%%  forum category</h1>
			<div style="color: red; display: none" id="template-id-stub-form-gerror-container">
				Please correct the following errors:
				<div id="template-id-stub-form-gerror-list">
				</div>
			</div>
			<table class="form">
				<tr>
					<td>
						Category name:
					</td>
					<td>
						<input  class="text" type="text" name="category-name" id="template-id-stub-gcategory-name" size="40" maxlength="50"/>
					</td>
				</tr>
				<tr>
					<td>
						Short description:
					</td>
					<td>
						<textarea name="category-description" id="template-id-stub-gcategory-description" cols="50" rows="3"></textarea>
					</td>
				</tr>
				<tr>
					<td>
						Posts structure - max nesting level: 
					</td>
					<td>
						<select name="gcategory-structure" id="template-id-stub-gcategory-structure">
							<option value="">forum default</option>
							<option value="0">0 (flat/linear)</option>
							<option value="1">1</option>
							<option value="2">2</option>
							<option value="3">3</option>
							<option value="4">4</option>
							<option value="5">5</option>
							<option value="6">6</option>
							<option value="7">7</option>
							<option value="8">8</option>
							<option value="9">9</option>
							<option value="10">10</option>
						</select>
					</td>
				</tr>
			</table>
						<input type="hidden" id="template-id-stub-group-index"/>
						<input type="hidden" id="template-id-stub-category-index"/>
			
		</div>
		<div class="button-bar">
			<a href="javascript:;"  onclick="OZONE.dialog.cleanAll()">cancel</a>
			<a href="javascript:;" onclick="WIKIDOT.modules.ManageSiteForumLayoutModule.listeners.saveCategory(event)">save</a>
		</div>
	</div>
</div>

