<template>
  <v-container id="xterm-container" fluid class="no-padding">
    <v-row>
      <v-col sm="12" md="12" lg="12">
          <v-btn variant="tonal" block v-if="!isWsConnected()" @click="terminalConnect">
            Connect
          </v-btn>
          <div style="height:10px;"></div>
          <v-btn variant="outlined" color="red" block v-if="isWsConnected()" @click="terminalDisconnect">
            Disconnect
          </v-btn>
      </v-col>
      <v-col sm="12" md="12" lg="12" class="no-padding">
        <div id="xterm"></div> 
      </v-col>
    </v-row>
  </v-container>
</template>

<style>
#xterm-container #xterm_terminals{
  padding:0px;
}
#xterm-container .v-row{
  margin:0px;
  background-color:#1b1b1b;
  color:#FFF;
}
.xterm .xterm-viewport{
  overflow-y: auto;
}
.no-padding{
  padding:0px;
}
body{
  padding:0px;
  margin:0px;
}
#xterm {
  border-top:5px solid #757575;
  padding:0px;
  min-height:100vh;
  width:100%;
}

#xterm {
  font-family: system-ui, -apple-system, BlinkMacSystemFont, 'Segoe UI', Roboto, Oxygen, Ubuntu, Cantarell, 'Open Sans', 'Helvetica Neue', sans-serif;
  color:#FFF;
  text-align: left;
  margin:0px;
  padding:3px;
  height:auto;
  width:100%;
}
</style>
<script>
import { AnsiUp } from 'ansi_up';
import {Terminal } from '@xterm/xterm';
import '@xterm/xterm/css/xterm.css';
import { FitAddon } from '@xterm/addon-fit';
import { CanvasAddon } from '@xterm/addon-canvas';

const ansi_convert = new AnsiUp();
const xterm = new Terminal();
const fitAddon = new FitAddon();
const canvasAddon = new FitAddon();
ansi_convert.use_classes = true;
export default{
  props:{
    name: {
      type: String,
      default: null
    }
  },
  mounted(){
    this.xterm_dom = document.getElementById('xterm');
    xterm.open(this.xterm_dom);
    xterm.loadAddon(fitAddon);
    xterm.loadAddon(canvasAddon);
    xterm.options = this.xterm_options;
    xterm.options.theme.background = "1b1b1b";
    xterm.onKey(this.onXtermKey);
    window.onresize = ()=> {
      this.computeXtermResize();
    };
  },
  methods:{
    terminalDisconnect(){
      let connection = this.ws_config.connection;
      if(connection == null) {
        return false;
      }
      connection.close();
      console.log("disconnect");
      return true;
    },
    terminalConnect(){
      this.computeXtermResize().then(()=>{
        this.wsConnect();
      })
    },
    wsConnect(){
      let config = this.ws_config;
      let params = new URLSearchParams();
      params.append("terminal_name",config.terminal_name);
      params.append("terminal_rows",config.terminal_rows);
      params.append("terminal_cols",config.terminal_cols);
      this.ws_config.loading = true;
      let ws = new WebSocket(config.url+"?"+params.toString());
      ws.onmessage = this.wsOnMessage;
      ws.onopen = this.wsOnOpen;
      ws.onclose = this.wsOnClose;
      this.ws_config.connection = ws;
    },
    isWsConnected(){
      let connection = this.ws_config.connection;
      if(connection == null) {
        return false;
      }
      return (connection.OPEN) ? true : false;
    },
    wsOnClose(e){
      console.log("wsOnClose",e);
      this.ws_config.connection = null;
      xterm.reset();
    },
    wsOnOpen(e){
      this.ws_config.loading = false;
      this.sendHeartBeat();
    },
    sendHeartBeat(){
      let connection = this.ws_config.connection;
      if(connection != null) {
        console.log(connection);
        if(connection.OPEN || connection.CONNECTING){
          this.ws_config.connection.send("__HEARTBEAT__");
          setTimeout(()=> {
            this.sendHeartBeat();
          },15000);
        }
      }
    },
    wsOnMessage(evt){
      let blob = evt.data;
      let reader = new FileReader();
      reader.onloadend = (e) => {
        xterm.write(reader.result);
        this.ws_config.loading = false;
      };
      reader.readAsText(blob);
    },
    computeXtermResize(){
      return new Promise((resolved)=>{
        this.xterm_resize_timer = clearTimeout(this.xterm_resize_timer);
        this.xterm_resize_timer = setTimeout(()=>{
          fitAddon.fit();
          this.ws_config.terminal_cols = xterm.cols;
          this.ws_config.terminal_rows = xterm.rows;
          
          resolved({"col":xterm.cols, "rows":xterm.rows});
        },1000);
      })
    },
    onXtermKey(e){
      this.ws_config.connection.send(e.key);
    },
    scrollToBottom(){
      let x = document.getElementById("terminal");
      x.scrollTop = x.scrollHeight + 100;
    },
    getSelectedTerminal(){
      let selected = this.xterm_terminals.find((item) => item.active == true );
      return (selected == undefined) ? null : selected ;
    },
    onSelectTerminal(id){
      this.xterm_terminals = this.xterm_terminals.map((item)=> {
        if(item.id == id) {
          item.active = true;
        }
        else{
          item.active = false;
        }
        return item;
      })
    }
  },
  data(){
    return {
      ws_config: {
        loading:false,
        connection:null,
        url: "http://localhost:3032/terminal",
        terminal_name:"web-terminal-in-rust",
        terminal_rows:50,
        terminal_cols:10,
      },
      xterm_dom: null,
      xterm_options: {
        fontSize: 20,
        theme: {
          background: "#1b1b1b",
        },
        fontFamily: "system-ui, -apple-system, BlinkMacSystemFont, 'Segoe UI', Roboto, Oxygen, Ubuntu, Cantarell, 'Open Sans', 'Helvetica Neue', sans-serif;"
      },
      lines:[],
      ws:null,
      command:"",
      loading:false,
      command_timer:null,
      xterm_resize_timer:null,
    }
  }
};

</script>
