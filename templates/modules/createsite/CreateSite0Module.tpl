{loadmacro set="Forms"}

{macro name="printErrorMessages" messages=$data_errorMessages}
{macro name="printFormErrors" form=$form}


<form id="new-site1" action="" method="post">
{macro name="defformhelps" form=$form}
{$form->declarations()}
<input type="hidden" name="action" value="CreateSiteAction"/>
<input type="hidden" name="event" value="processCreateForm"/>

{macro name="formtablestart" title="new wiki site" id="nw1table"}
{macro name="formrow1" form=$form fieldname="name"} 
{macro name="formrow1" form=$form fieldname="unixname"}
{macro name="formrow1" form=$form fieldname="subtitle"}
{macro name="formrow1" form=$form fieldname="description"}   

{*
<tr id="showmoreopts"><td colspan="2">
<a href="javascript:void(0)" onclick="newWikiShowMoreOptions()">show more options &darr;</a>
</td></tr>
{macro name="formtablehr"}
{macro name="formrow1" form=$form fieldname="licence" style="display: none"} 
*}
{macro name="formrow1" form=$form fieldname="licence" } 
{macro name="formrow1" form=$form fieldname="licence-text" idr="other-licence-row"} 
{macro name="formtablehr"}
<tr class="buttons">
		<td colspan="2">
				<input type="button" name="cancel" value="cancel" onclick="WIKIDOT.modules.CreateSite0Module.listeners.cancelClick(event)"/>
				<input type="button" name="next" value="next &rarr;" onclick="WIKIDOT.modules.CreateSite0Module.listeners.nextClick(event)"/>
		</td>

	</tr>
{*{macro name="formbuttons1" submit_event="finalize"} *}
{macro name="formtableend"}




</form>

new wiki module