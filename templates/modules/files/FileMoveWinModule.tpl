<h1>{t}Move file{/t}</h1>

<table class="form">
	<tr>
		<td>
			{t}File name{/t}: 
		</td>
		<td>
			<strong>{$file->getFilename()|escape}</strong>
		</td>
	</tr>
	<tr>
		<td>
			{t}Current page{/t}: 
		</td>
		<td>
			{$page->getUnixName()}
		</td>
	</tr>
	<tr>
		<td>
			{t}Destination page{/t}: 
		</td>
		<td>
			<div id="file-move-page-div" class="autocomplete-container">
				<input type="text" id="file-move-page" size="30" class="autocomplete-input text"/>
				<div id="file-move-page-autocomplete" class="autocomplete-list"></div>
			</div>
		</td>
	</tr>
</table>

<div id="file-move-error" class="error-block" style="display: none"></div>
