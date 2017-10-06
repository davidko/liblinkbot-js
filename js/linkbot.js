
var DaemonSingleton = (function() {
	var instance;
 
    function createInstance() {
        var object = Module.ccall('daemon_new', '*', 
                [], []);
        return object;
    }
 
    return {
        getInstance: function () {
            if (!instance) {
                instance = createInstance();
            }
            return instance;
        }
    };
})();

var Robot = function(context) {
    var self = this;
    self.context = context;

    self.setLedColor = function(red, green, blue) {
        return new Promise( function(resolve, reject) {
            var cb_ptr = Module.Runtime.addFunction(resolve);
            Module.ccall('robot_set_led_color', 'number',
                    ['*', 'number', 'number', 'number', '*'],
                    [self.context,
                     red,
                     green,
                     blue,
                     cb_ptr]);
        });
    }
};

var DaemonProxy = function() {
    var self = this;
    self.instance = DaemonSingleton.getInstance();

    self.setWriteCallback = function(callback) {
        var cb_wrapper = function(vec) {
            var len = Module.getValue(vec+4, 'i32');
            var slice_ptr = Module.getValue(vec, '*');
            var buffer = new Uint8Array(len);
            for(var i = 0; i < len; i++) {
                var value = Module.getValue(slice_ptr+i, 'i8');
                buffer[i] = value;
            }
            console.log("Daemon proxy sending bytes:");
            console.log(buffer);
            callback(buffer);
        };
        var cb_ptr = Module.Runtime.addFunction(cb_wrapper);
        Module.ccall('daemon_set_write_callback', 'number',
                ['*', '*'],
                [self.instance, cb_ptr]);
    }

    self.deliver = function(buffer /* Uint8Array or Buffer */) {
        Module.ccall('daemon_deliver',
                   'number',
                   ['*', 'array', 'number'],
                   [self.instance, buffer, buffer.length]
                );

    };

    self.getRobot = function(serialId) {
        return new Promise( function(resolve, reject) {
            var handle = Module.ccall('daemon_get_robot', '*',
                    ['*', 'string'], [self.instance, serialId]);
            var cb_wrapper = function() {
                resolve(new Robot(handle));
            };
            var cb_ptr = Module.Runtime.addFunction(cb_wrapper);
            Module.ccall('daemon_connect_robot', 'number',
                    ['*', '*', 'string', '*'],
                    [self.instance, handle, serialId, cb_ptr]);
        });
    };
}


