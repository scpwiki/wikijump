<h1>Navigation elements</h1>

<p>
	You can choose which navigation elements (<em>top-bar</em> and <em>side-bar</em>) 
	should appear in pages within a specified category.
</p>
	
	<table class="form">
		<tr>
			<td>
				Choose the category:
			</td>
			<td>
				<select name="category" size="15" id="sm-nav-cats">
					{foreach from=$categories item=category}
						<option value="{$category->getCategoryId()}" style="padding: 0 1em" {if $category->getName()=="_default"}selected="selected"{/if}>{$category->getName()|escape}</option>
					{/foreach}
				</select>
			</td>
			<td style="padding-left: 2em;">
				<div id="sm-nav-noind">
					No individual nav elements <input class="checkbox" type="checkbox" id="sm-nav-noin"/>
				</div>
			</td>
		</tr>
	</table>
	
	<div id="sm-nav-list">
		<table class="form">
			<tr>
				<td>
					top-bar:
				</td>
				<td>
					<div class="autocomplete-container" style="width: 20em">
						<input type="text" id="sm-nav-top-bar" class="autocomplete-input text" name="top_bar" size="35"/>
						<div id="sm-nav-top-bar-list" class="autocomplete-list"></div>
					</div>
				</td>
			</tr>
			<tr>
				<td>
					side-bar:
				</td>
				<td>
					<div class="autocomplete-container"  style="width: 20em">
						<input type="text" id="sm-nav-side-bar" class="autocomplete-input text"  name="side_bar" size="35"/>
						<div id="sm-nav-side-bar-list" class="autocomplete-list"></div>
					</div>
				</td>
			</tr>
		</table>
	</div>
	
	<div class="buttons">
		<input type="button" value="cancel" id="sm-nav-cancel"/>
		<input type="button" value="save changes" id="sm-nav-save"/>
	</div>

<p>
	NOTE: if the chosen pages do not exist no navigation elemen will be displayed.
</p>
<p>
	BUT if you really want to get rid of the side element you should also choose a proper 
	theme without the side bar. Most of the available themes have such a variant.
</p>


