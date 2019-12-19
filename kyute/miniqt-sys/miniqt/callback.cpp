#include "callback.hpp"


MQCallback::MQCallback(uintptr_t data0, uintptr_t data1,
                       MQCallback_ptr callback)
    : data0{data0}, data1{data1}, callback{callback} 
{
}

void MQCallback::trigger() { callback(data0, data1); }

MQCallback_QString::MQCallback_QString(uintptr_t data0, uintptr_t data1,
                                       MQCallback_QString_ptr callback)
    : data0{data0}, data1{data1}, callback{callback} {}

void MQCallback_QString::trigger(const QString &str) {
  callback(data0, data1, str);
}

MQCallback_int::MQCallback_int(uintptr_t data0, uintptr_t data1,
                               MQCallback_int_ptr callback)
    : data0{data0}, data1{data1}, callback{callback} {}

void MQCallback_int::trigger(int i) { callback(data0, data1, i); }

/*
void MQCallback_destructor(MQCallback* callback) {
	callback->~MQCallback();
}

void MQCallback_int_destructor(MQCallback_int* callback) {
	callback->~MQCallback_int();
}

void MQCallback_QString_destructor(MQCallback_QString* callback) {
	callback->~MQCallback_QString();
}*/

QObject* MQCallback_new(uintptr_t data0, uintptr_t data1, MQCallback_ptr callback) {
	return new MQCallback(data0, data1, callback);
}

QObject* MQCallback_int_new(uintptr_t data0, uintptr_t data1, MQCallback_int_ptr callback) {
	return new MQCallback_int(data0, data1, callback);
}

QObject* MQCallback_QString_new(uintptr_t data0, uintptr_t data1, MQCallback_QString_ptr callback) {
	return new MQCallback_QString(data0, data1, callback);
}

/*
void MQCallback_delete(MQCallback *w) {
  auto d = (QObject *)w;
  delete d;
}
*/
