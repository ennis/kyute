#pragma once
#include "miniqt.hpp"
#include <QObject>
#include <cstdint>
#include <iostream>
#include <memory>

//---------------------------------------------------------------------------
class MQCallback : public QObject {
	Q_OBJECT
public:
	MQCallback(uintptr_t data0, uintptr_t data1, MQCallback_ptr callback);

public Q_SLOTS:
	void trigger();

private:
	uintptr_t data0;
	uintptr_t data1;
	MQCallback_ptr callback;
};

class MQCallback_QString : public QObject {
	Q_OBJECT
public:
	MQCallback_QString(uintptr_t data0, uintptr_t data1,
		MQCallback_QString_ptr callback);


public Q_SLOTS:
	void trigger(const QString &str);

private:
	uintptr_t data0;
	uintptr_t data1;
	MQCallback_QString_ptr callback;
};

class MQCallback_int : public QObject {
	Q_OBJECT
public:
	MQCallback_int(uintptr_t data0, uintptr_t data1, MQCallback_int_ptr callback);

public Q_SLOTS:
	void trigger(int i);

private:
	uintptr_t data0;
	uintptr_t data1;
	MQCallback_int_ptr callback;
};

/*
#define MQT_DEFINE_VOID_SIGNAL(ty, signame)                                    \
 MQCallback *ty##_##signame##(ty * this_, uintptr_t data0,         \
                                          uintptr_t data1,                     \
                                          MQCallbackFn_void callback) {        \
    auto cb = new miniqt::Callback_void(data0, data1, callback);               \
    this_->connect(this_, &ty::signame, cb, &miniqt::Callback_void::trigger);  \
    return cb;                                                                 \
  }

#define MQT_DEFINE_SIGNAL(ty, signame, argty)                                  \
MQCallback *ty##_##signame##(ty * this_, uintptr_t data0,         \
                                          uintptr_t data1,                     \
                                          MQCallbackFn_##argty callback) {     \
    auto cb = new miniqt::Callback_##argty(data0, data1, callback);            \
    this_->connect(this_, &ty::signame, cb,                                    \
                   &miniqt::Callback_##argty::trigger);                        \
    return cb;                                                                 \
  }
  */