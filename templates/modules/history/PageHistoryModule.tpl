<h1>{t}Page history of changes{/t}</h1>

<div style="float:right">
	<a href="javascript:;" onclick="WIKIDOT.modules.PageHistoryModule.listeners.watchPage(event)">{t}add to watched{/t}</a>
</div>

<form id="history-form-1">
	<table class="form">
		
		<tr>
			<td>
				{t}Show page changes{/t}:
			</td>
			<td>	
				<input class="checkbox" type="checkbox" id="rev-type-all" checked="checked"/>&nbsp;{t}ALL{/t}<br/>
				<input class="checkbox" type="checkbox" id="rev-type-source"/>&nbsp;{t}source changes{/t}<br/>
				<input class="checkbox" type="checkbox" id="rev-type-title"/>&nbsp;{t}title{/t}<br/>
				<input class="checkbox" type="checkbox" id="rev-type-move"/>&nbsp;{t}move/rename{/t}<br/>
				<input class="checkbox" type="checkbox" id="rev-type-meta"/>&nbsp;{t}meta data{/t}<br/>
				<input class="checkbox" type="checkbox" id="rev-type-files"/>&nbsp;{t escape=no}attachments&nbsp;(files){/t}
			</td>
		</tr>
		<tr>
			<td>
				{t}Revisions per page{/t}:
			</td>
			<td>
				<select id="h-perpage">
					<option value="10">10</option>
					<option value="20" selected="selected">20</option>
					<option value="50">50</option>
					<option value="100">100</option>
					<option value="200">200</option>
				</select>
			</td>
		</tr>
	</table>
	<div class="buttons">
		<input type="button" value="{t}update list{/t}" onclick="WIKIDOT.modules.PageHistoryModule.listeners.updateList(event)"/>
	</div>
	
	<input class="button" type="button" name="compare" id="history-compare-button" value="{t}compare versions{/t}"/> 
	
	<div id="revision-list">
		loading revision list...
	</div>
	{*<input type="button" name="compare" onclick="wikiPageHistory2Click()" value="compare versions"/>*}
</form>

<div id="history-subarea" style="display: none">
</div>